import pillow_avif
from pgvector.asyncpg.register import register_vector
import asyncio
import asyncpg
import os
import torch
from dotenv import load_dotenv
from transformers import (
    CLIPModel,
    CLIPProcessor,
    BlipProcessor,
    BlipForConditionalGeneration
)
from PIL import Image
import hashlib

async def main():
    load_dotenv()
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set.")

    conn = await asyncpg.connect(database_url)
    print("Connected to Postgres")

    # Register the VECTOR type
    await register_vector(conn)

    # Load the models
    clip_model_name = "openai/clip-vit-base-patch32"
    clip_model = CLIPModel.from_pretrained(clip_model_name)
    clip_processor = CLIPProcessor.from_pretrained(clip_model_name)
    clip_model = clip_model.eval()
    device = "cuda" if torch.cuda.is_available() else "cpu"
    clip_model = clip_model.to(device)

    blip_model_name = "Salesforce/blip-image-captioning-large"
    blip_processor = BlipProcessor.from_pretrained(blip_model_name)
    blip_model = BlipForConditionalGeneration.from_pretrained(blip_model_name).to(device).eval()

    while True:
        # 1) Find an unfulfilled request
        row = await conn.fetchrow("""
            SELECT r.id, r.file_id, r.request_type, r.model, f.path
            FROM files.requests r
            JOIN files.files f ON r.file_id = f.id
            WHERE r.fulfilled_at IS NULL
              AND r.error_message IS NULL
            ORDER BY r.id
            LIMIT 1
        """)
        if not row:
            print("No unfulfilled requests. Sleeping 5 seconds...")
            await asyncio.sleep(5)
            continue

        req_id = row["id"]
        file_id = row["file_id"]
        req_type = row["request_type"]
        model = row["model"] or ""
        file_path = row["path"]

        print(f"Processing request {req_id}: file_id={file_id}, type={req_type}, model={model}")

        try:
            if req_type == "embed":
                # If model is blank, let's assume "clip-vit-base-patch32"
                emb_model = model if model else clip_model_name
                # We'll just do the embedding with the CLIP code
                # Real code might handle multiple model strings
                embedding = await generate_clip_embedding(file_path, clip_model, clip_processor, device)
                # Insert into files.embeddings
                await conn.execute("""
                    INSERT INTO files.embeddings_512(file_id, model, embedding)
                    VALUES ($1, $2, $3)
                """, file_id, emb_model, embedding)
                # Mark request as fulfilled
                await conn.execute("""
                    UPDATE files.requests
                    SET fulfilled_at = now()
                    WHERE id = $1
                """, req_id)

            elif req_type == "caption":
                cap_model = model if model else blip_model_name
                # Use BLIP
                caption = generate_blip_caption(file_path, blip_model, blip_processor, device)
                # Insert into files.captions
                await conn.execute("""
                    INSERT INTO files.captions(file_id, model, caption)
                    VALUES ($1, $2, $3)
                """, file_id, cap_model, caption)
                # Mark request as fulfilled
                await conn.execute("""
                    UPDATE files.requests
                    SET fulfilled_at = now()
                    WHERE id = $1
                """, req_id)

            else:
                raise ValueError(f"Unsupported request type: {req_type}")

        except Exception as e:
            # Store the error in requests.error_message
            print(f"Error processing request {req_id}: {e}")
            await conn.execute("""
                UPDATE files.requests
                SET error_message = $2
                WHERE id = $1
            """, req_id, str(e))

        # Continue to the next request

async def generate_clip_embedding(file_path, clip_model, clip_processor, device):
    from PIL import Image
    image = Image.open(file_path).convert("RGB")
    inputs = clip_processor(images=image, return_tensors="pt").to(device)
    with torch.no_grad():
        image_embeds = clip_model.get_image_features(**inputs)
        image_embeds = image_embeds / image_embeds.norm(p=2, dim=-1, keepdim=True)
    # Convert to Python list float array
    embedding_list = image_embeds.squeeze(0).cpu().tolist()
    return embedding_list  # asyncpg can store as JSON or we can store in a vector column

def generate_blip_caption(file_path, blip_model, blip_processor, device):
    image = Image.open(file_path).convert("RGB")
    inputs = blip_processor(image, return_tensors="pt").to(device)
    with torch.no_grad():
        out = blip_model.generate(**inputs, max_length=50)
    caption = blip_processor.decode(out[0], skip_special_tokens=True)
    return caption

if __name__ == "__main__":
    print("Launching...")
    asyncio.run(main())
