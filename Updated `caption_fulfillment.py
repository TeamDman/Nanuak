import os
import json
import asyncio
import asyncpg
from dotenv import load_dotenv
import torch
from PIL import Image
from io import BytesIO
import requests
from transformers import BlipProcessor, BlipForConditionalGeneration

async def main():
    # 1) Load environment variables
    load_dotenv()  # loads DATABASE_URL from .env
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set in the environment")

    conn = await asyncpg.connect(database_url)
    print("Connected to the database.")

    # Listen for 'caption_request'
    await conn.add_listener("caption_request", handle_caption_request)
    print("Listening on 'caption_request'...")

    try:
        await asyncio.Future()  # keep running
    except asyncio.CancelledError:
        pass
    finally:
        await conn.remove_listener("caption_request", handle_caption_request)
        await conn.close()
        print("Shut down listener.")

async def handle_caption_request(conn, pid, channel, payload):
    """
    Example: we get NOTIFY caption_request, '{ "image_path": "/path/to.jpg", "request_id": 123 }'
    We'll generate a mock caption, then notify 'caption_response' with the result.
    Also optionally insert directly into caption_responses.
    """
    print(f"[caption_request] Received: {payload}")
    try:
        data = json.loads(payload)
    except json.JSONDecodeError:
        print("Invalid JSON payload for caption_request.")
        return

    request_id = data.get("request_id")
    image_path = data.get("image_path")
    if request_id is None or not image_path:
        print("Missing request_id or image_path.")
        return

    # In a real system, you'd load the image and run a caption model. We'll just do a random string.
    try:
        resp = requests.get(image_path, stream=True)
        resp.raise_for_status()
        image = Image.open(BytesIO(resp.raw.read())).convert("RGB")

        inputs = blip_processor(image, return_tensors="pt").to(device)
        with torch.no_grad():
            out = blip_model.generate(**inputs, max_length=50)
        caption_text = blip_processor.decode(out[0], skip_special_tokens=True)

    except requests.exceptions.RequestException as e:
        print(f"Error fetching image: {e}")
        caption_text = f"Error generating caption for {image_path}: Could not fetch image."
        # Handle the error appropriately, e.g., log it, return an error response, etc.
        return  # Or continue with a default caption

    except Exception as e:
        print(f"Error generating caption: {e}")
        caption_text = f"Error generating caption for {image_path}: {e}"
        return  # Or continue with a default caption


    # Option A: Insert directly into DB
    insert_sql = """
        INSERT INTO public.caption_requests(image_path, caption_text)
        VALUES ($1, $2)
        ON CONFLICT (image_path) DO UPDATE SET caption_text=$2, completed=TRUE
    """
    try:
        await conn.execute(insert_sql, image_path, caption_text)
        print(f"Inserted new caption request for path={image_path}")
    except Exception as e:
        print(f"Error inserting caption request: {e}")
        return

    # Option B: Also broadcast via caption_response channel
    # So that any "logger" service can do additional steps, if needed.
    resp_payload = json.dumps({
        "caption_request_id": request_id,
        "caption_text": caption_text
    })
    await conn.execute("SELECT pg_notify($1, $2)", "caption_response", resp_payload)
    print(f"NOTIFY caption_response => {resp_payload}")
