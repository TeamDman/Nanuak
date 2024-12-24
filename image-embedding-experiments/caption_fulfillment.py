# caption_fulfillment.py

import os
import json
import asyncio
import asyncpg
from dotenv import load_dotenv
import random
import torch
from PIL import Image
from io import BytesIO
import requests
from transformers import BlipProcessor, BlipForConditionalGeneration


async def main():
    load_dotenv()
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set")

    global device, blip_processor, blip_model  # Make these available within the module
    device = "cuda" if torch.cuda.is_available() else "cpu"
    blip_processor = BlipProcessor.from_pretrained("Salesforce/blip-image-captioning-large")
    blip_model = BlipForConditionalGeneration.from_pretrained("Salesforce/blip-image-captioning-large").to(device)

    global device, blip_processor, blip_model  # Make these available within the module
    device = "cuda" if torch.cuda.is_available() else "cpu"
    blip_processor = BlipProcessor.from_pretrained("Salesforce/blip-image-captioning-large")
    blip_model = BlipForConditionalGeneration.from_pretrained("Salesforce/blip-image-captioning-large").to(device)

    conn = await asyncpg.connect(database_url)
    print("Caption Fulfillment & Logger service connected. BLIP model loaded.")

    # Listen for requests
    await conn.add_listener("caption_request", handle_caption_request)
    # Listen for responses to log them
    await conn.add_listener("caption_response", handle_caption_response)
    print("Listening for 'caption_request' and 'caption_response'...")

    try:
        await asyncio.Future()
    except asyncio.CancelledError:
        pass
    finally:
        await conn.close()
        print("Service shutting down.")

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
        INSERT INTO captions.caption_responses (caption_request_id, caption_text)
        VALUES ($1, $2)
        RETURNING id
    """
    try:
        row = await conn.fetchrow(insert_sql, request_id, caption_text)
        resp_id = row["id"]
        print(f"Inserted new caption_responses row id={resp_id} for request_id={request_id}")
    except Exception as e:
        print(f"Error inserting caption response: {e}")
        return

    # Option B: Also broadcast via caption_response channel
    # So that any "logger" service can do additional steps, if needed.
    resp_payload = json.dumps({
        "caption_request_id": request_id,
        "caption_text": caption_text
    })
    await conn.execute("SELECT pg_notify($1, $2)", "caption_response", resp_payload)
    print(f"NOTIFY caption_response => {resp_payload}")

async def handle_caption_response(conn, pid, channel, payload):
    """
    If you'd rather do the insertion here (rather than in handle_caption_request),
    you can handle it in a separate service.
    For demonstration, we'll just log it.
    """
    print(f"[caption_response] Received: {payload}")
    # We already inserted into DB in handle_caption_request. 
    # So maybe we do something else, like notify a user or update a log.
    # We'll just print in this demo.
    # If you want to store in a table, do it here the same way we do in handle_caption_request.
    pass

if __name__ == "__main__":
    asyncio.run(main())
