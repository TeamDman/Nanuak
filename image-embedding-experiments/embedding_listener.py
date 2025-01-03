import os
import json
import asyncio
import asyncpg
import hashlib
import numpy as np
from dotenv import load_dotenv
import torch
from PIL import Image
from io import BytesIO
import requests
from transformers import CLIPProcessor, CLIPModel

async def main():
    # 1) Load environment variables
    load_dotenv()  # e.g., picks up DATABASE_URL from .env
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set in the environment")

    # 2) Connect to Postgres
    # Load CLIP model and processor
    global clip_model, clip_processor, device  # Make these globally available
    model_ckpt = "openai/clip-vit-base-patch32"
    clip_model = CLIPModel.from_pretrained(model_ckpt)
    clip_processor = CLIPProcessor.from_pretrained(model_ckpt)
    device = "cuda" if torch.cuda.is_available() else "cpu"
    clip_model = clip_model.to(device).eval()

    conn = await asyncpg.connect(database_url)
    print("Connected to the database. CLIP model loaded.")


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
    # Fetch the image (assuming it's a URL for now.  Adjust if needed)
    try:
        resp = requests.get(path, stream=True)  # Use stream=True for large images
        resp.raise_for_status()  # Raise an exception for bad status codes (4xx or 5xx)
        image = Image.open(BytesIO(resp.raw.read())).convert("RGB")

        # Preprocess and generate embedding using CLIP
        inputs = clip_processor(images=image, return_tensors="pt").to(device)
        with torch.no_grad():
            image_embeds = clip_model.get_image_features(**inputs)
            image_embeds = image_embeds / image_embeds.norm(p=2, dim=-1, keepdim=True)
        embedding_vector = image_embeds.cpu().squeeze(0).numpy()

    except requests.exceptions.RequestException as e:
        print(f"Error fetching image: {e}")
        return
    except Exception as e:
        print(f"Error generating embedding: {e}")
        return

    hash_value = compute_file_hash(path, hash_algorithm)


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
    return hash_value # Return the hash

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
