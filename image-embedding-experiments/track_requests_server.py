import os
import json
import asyncio
import asyncpg
from dotenv import load_dotenv

async def main():
    load_dotenv()  # loads DATABASE_URL from .env
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set in the environment")

    conn = await asyncpg.connect(database_url)
    print("Connected to the database.")

    # Listen for 'embedding_inserted'
    await conn.add_listener("embedding_inserted", handle_embedding_inserted)
    print("Listening on 'embedding_inserted'...")

    try:
        await asyncio.Future()  # keep running
    except asyncio.CancelledError:
        pass
    finally:
        await conn.remove_listener("embedding_inserted", handle_embedding_inserted)
        await conn.close()
        print("Shut down listener.")

async def handle_embedding_inserted(conn, pid, channel, payload):
    """
    Triggered whenever we get NOTIFY embedding_inserted, '<payload>'.
    Expects JSON like: {"path": "...", "hash_value": "...", "hash_algorithm": "..."}.
    We'll store a record in image_embeddings.embedding_requests.
    """
    print(f"[EVENT] Received on channel '{channel}': {payload}")
    try:
        data = json.loads(payload)
    except json.JSONDecodeError:
        print(f"Invalid JSON payload: {payload}")
        return

    image_path = data.get("path")
    image_hash = data.get("hash_value")
    image_hash_algo = data.get("hash_algorithm")
    if not (image_path and image_hash and image_hash_algo):
        print("Missing required fields in payload.")
        return

    # Insert into the embedding_requests table
    sql = """
    INSERT INTO image_embeddings.embedding_requests (image_path, image_hash, image_hash_algo)
    VALUES ($1, $2, $3)
    """
    try:
        await conn.execute(sql, image_path, image_hash, image_hash_algo)
        print(f"Inserted record for path='{image_path}' hash='{image_hash}'.")
    except Exception as e:
        print(f"Error inserting record: {e}")

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Exiting listener.")
