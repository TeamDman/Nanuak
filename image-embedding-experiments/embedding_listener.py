import os
import json
import asyncio
import asyncpg
import hashlib
import numpy as np
from dotenv import load_dotenv

async def main():
    # 1) Load environment variables
    load_dotenv()  # e.g., picks up DATABASE_URL from .env
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set in the environment")

    # 2) Connect to Postgres
    conn = await asyncpg.connect(database_url)
    print("Connected to the database.")

    # 3) Listen for 'embedding_request' channel
    await conn.add_listener("embedding_request", handle_embedding_request)
    print("Listening on 'embedding_request'...")

    # Keep running forever unless forcibly stopped
    try:
        await asyncio.Future()
    except asyncio.CancelledError:
        pass
    finally:
        # 4) Cleanup on exit
        await conn.remove_listener("embedding_request", handle_embedding_request)
        await conn.close()
        print("Shut down listener.")

async def handle_embedding_request(conn, pid, channel, payload):
    """
    Triggered whenever a NOTIFY embedding_request, '<payload>' occurs in Postgres.
    The `payload` is a string, which we assume is JSON containing {"path": "..."}.
    """
    print(f"[EVENT] Received on channel '{channel}': {payload}")
    try:
        data = json.loads(payload)
    except json.JSONDecodeError:
        print(f"Invalid JSON payload: {payload}")
        return

    path = data.get("path")
    if not path:
        print("Missing 'path' in payload.")
        return

    # 1) Compute a hash for the file
    #    (In real usage, handle errors if file doesn't exist, etc.)
    hash_algorithm = "sha256"
    hash_value = compute_file_hash(path, hash_algorithm)

    # 2) Generate a dummy embedding. In real usage, call your embedding model.
    embedding_dim = 768
    embedding_vector = np.random.randn(embedding_dim).astype(np.float32)

    # 3) Insert into DB
    try:
        await insert_embedding(conn, hash_value, hash_algorithm, embedding_vector)
        print(f"Inserted embedding for path={path}, hash_value={hash_value}.")
    except Exception as e:
        print(f"Error inserting embedding: {e}")
        return

    # 4) Notify 'embedding_inserted'
    event_payload = {
        "path": path,
        "hash_value": hash_value,
        "hash_algorithm": hash_algorithm,
    }
    await conn.execute(
        "SELECT pg_notify($1, $2)",
        "embedding_inserted",
        json.dumps(event_payload)
    )
    print(f"NOTIFY embedding_inserted with {event_payload}")

def compute_file_hash(path, algorithm="sha256"):
    """Compute a SHA256 hash for the given file path."""
    hasher = hashlib.new(algorithm)
    with open(path, "rb") as f:
        chunk = f.read(8192)
        while chunk:
            hasher.update(chunk)
            chunk = f.read(8192)
    return hasher.hexdigest()

async def insert_embedding(conn, hash_value, hash_algorithm, embedding):
    """
    Insert a new record into image_embeddings.embeddings with a vector(768).
    We'll store the embedding as parameterized float[] in asyncpg,
    and cast to vector(768).
    """
    # Convert from np.array to Python list
    embedding_list = embedding.tolist()  # shape: (768,)

    # Postgres expects the vector data in a special format if you want to CAST to vector(768).
    # Alternatively, use a float4[] column if you prefer. For vector(768),
    # we can pass as 'ARRAY[...]' string, or rely on pgvector support in the driver.
    # For demonstration, let's build the 'ARRAY[...]' string.
    # E.g. "ARRAY[0.12, 0.34, ... ]::float4[]"

    array_string = "ARRAY[" + ",".join(str(x) for x in embedding_list) + "]::float4[]"

    sql = f"""
    INSERT INTO image_embeddings.embeddings(hash_value, hash_algorithm, embedding)
    VALUES ($1, $2, {array_string}::vector(768))
    ON CONFLICT (hash_value) DO NOTHING
    """
    await conn.execute(sql, hash_value, hash_algorithm)

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Exiting listener.")
