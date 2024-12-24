# caption_fulfillment.py

import os
import json
import asyncio
import asyncpg
from dotenv import load_dotenv
import random

async def main():
    load_dotenv()
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set")

    conn = await asyncpg.connect(database_url)
    print("Caption Fulfillment & Logger service connected.")

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
    caption_text = f"Mock caption for {os.path.basename(image_path)} with random number {random.randint(0,9999)}"

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
