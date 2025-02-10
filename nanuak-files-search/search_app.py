from fastapi import FastAPI, Query
app = FastAPI()

@app.on_event("startup")
async def startup():
    from dotenv import load_dotenv
    load_dotenv()
    import os
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        raise ValueError("DATABASE_URL must be set")
    import asyncpg
    app.state.pool = await asyncpg.create_pool(database_url)


    # load model + processor for text embeddings
    global clip_model, clip_processor, device
    import torch
    device = "cuda" if torch.cuda.is_available() else "cpu"
    model_ckpt = "openai/clip-vit-base-patch32"
    from transformers import CLIPProcessor, CLIPModel
    clip_model = CLIPModel.from_pretrained(model_ckpt).to(device)
    clip_processor = CLIPProcessor.from_pretrained(model_ckpt)

from pydantic import BaseModel
class SearchResult(BaseModel):
    file_id: int
    distance: float

@app.get("/search_embedding")
async def search_embedding(q: str = Query(...)):
    """
    1) We embed the query string via CLIP text
    2) We do a vector similarity search in Postgres
    3) Return top 10 results (file_id + distance)
    """
    # 1) compute text embedding
    inputs = clip_processor(text=[q], return_tensors="pt").to(device)
    import torch
    with torch.no_grad():
        text_embeds = clip_model.get_text_features(**inputs)
        text_embeds = text_embeds / text_embeds.norm(p=2, dim=-1, keepdim=True)
    embedding_vector = text_embeds.squeeze(0).cpu().tolist()

    # 2) query top 10 from files.embeddings_512
    async with app.state.pool.acquire() as conn:
        # Register the VECTOR type
        from pgvector.asyncpg.register import register_vector
        await register_vector(conn)
        
        sql = """
            SELECT file_id, (embedding <-> $1) as distance
            FROM files.embeddings_512
            ORDER BY embedding <-> $1
            LIMIT 10
        """
        rows = await conn.fetch(sql, embedding_vector)

    # 3) return results
    results = [
        SearchResult(file_id=r["file_id"], distance=r["distance"])
        for r in rows
    ]
    return results

# if you want to run
if __name__ == "__main__":
    import uvicorn
    uvicorn.run("search_app:app", host="127.0.0.1", port=9000, reload=True)
