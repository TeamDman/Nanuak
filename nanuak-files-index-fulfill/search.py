from pgvector.asyncpg.register import register_vector
import asyncio
import os
import asyncpg
import torch
from transformers import CLIPModel, CLIPProcessor
from dotenv import load_dotenv

async def main():
    load_dotenv()
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set in the environment.")

    # 1) Connect to Postgres
    conn = await asyncpg.connect(database_url)
    print("Connected to database.")

    # Register the VECTOR type
    await register_vector(conn)

    # 2) Load CLIP model/processor for text embeddings
    model_name = "openai/clip-vit-base-patch32"
    clip_model = CLIPModel.from_pretrained(model_name).eval()
    clip_processor = CLIPProcessor.from_pretrained(model_name)
    device = "cuda" if torch.cuda.is_available() else "cpu"
    clip_model = clip_model.to(device)

    # 3) Enter infinite loop
    while True:
        query = input("\nEnter a search term> ").strip()
        if not query:
            print("Empty query. Exiting.")
            break

        # 3a) Generate text embedding using CLIP
        text_inputs = clip_processor(text=[query], return_tensors="pt").to(device)
        with torch.no_grad():
            text_embeds = clip_model.get_text_features(**text_inputs)
            # Normalize the embedding
            text_embeds = text_embeds / text_embeds.norm(p=2, dim=-1, keepdim=True)

        # shape: (1, 512)
        embedding_vector = text_embeds.squeeze(0).cpu().tolist()  # shape: (512,)

        # 3b) Query Postgres using pgvector’s <-> operator
        # We'll retrieve top 10 most similar images to the text embedding
        sql = """
            SELECT f.id AS file_id,
                   f.path,
                   (e.embedding <-> $1) AS distance
            FROM files.files f
            JOIN files.embeddings_512 e ON e.file_id = f.id
            ORDER BY e.embedding <-> $1
            LIMIT 10
        """
        # asyncpg can accept a Python list for the vector param if you have pgvector configured
        rows = await conn.fetch(sql, embedding_vector)

        # 3c) Print results
        print(f"\nTop matches for \"{query}\":")
        for i, row in enumerate(rows, start=1):
            print(f"  {i}. file_id={row['file_id']}  path={row['path']}  distance={row['distance']:.4f}")

    await conn.close()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nExiting search tool.")
