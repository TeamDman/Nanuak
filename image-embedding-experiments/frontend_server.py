# main.py
import os
import uvicorn
from fastapi import FastAPI, Request, UploadFile, File, Form
from fastapi.responses import HTMLResponse, RedirectResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
import asyncpg
import shutil
import uuid

DATABASE_URL = os.getenv("DATABASE_URL", "postgres://postgres:password@localhost/nanuak")

app = FastAPI()

# Mount a "static" folder if you want custom JS/CSS assets
app.mount("/static", StaticFiles(directory="static"), name="static")

templates = Jinja2Templates(directory="templates")

@app.on_event("startup")
async def startup():
    # Create a global connection pool
    app.state.db_pool = await asyncpg.create_pool(DATABASE_URL)
    print("DB pool created.")

@app.on_event("shutdown")
async def shutdown():
    await app.state.db_pool.close()
    print("DB pool closed.")

@app.get("/", response_class=HTMLResponse)
async def index(request: Request):
    """
    Render the main page with a search bar and results area.
    We'll fetch the existing images & captions from DB to display.
    """
    # For demonstration, let's pull last 10 captions or so
    pool = request.app.state.db_pool
    async with pool.acquire() as conn:
        rows = await conn.fetch("""
            SELECT cr.id as request_id,
                   cr.image_path,
                   resp.caption_text
            FROM captions.caption_requests cr
            LEFT JOIN captions.caption_responses resp
               ON cr.id = resp.caption_request_id
            ORDER BY cr.request_time DESC
            LIMIT 10
        """)
    results = []
    for row in rows:
        results.append({
            "request_id": row["request_id"],
            "image_path": row["image_path"],
            "caption_text": row["caption_text"]
        })
    return templates.TemplateResponse("index.html", {"request": request, "results": results})

@app.post("/upload_image")
async def upload_image(file: UploadFile = File(...), text_query: str = Form(None)):
    """
    1. Save the uploaded file locally
    2. Insert a row in caption_requests
    3. Notify caption_request
    4. Return a redirect to home
    """
    # 1. Save file
    file_ext = os.path.splitext(file.filename)[1]
    unique_name = f"{uuid.uuid4()}{file_ext}"
    save_path = os.path.join("static", "uploads", unique_name)
    os.makedirs(os.path.dirname(save_path), exist_ok=True)

    with open(save_path, "wb") as f:
        shutil.copyfileobj(file.file, f)

    # 2. Insert caption request
    # (In real code, you might want to store the text_query somewhere if relevant.)
    # For now, we just store the image_path
    # The request_id is returned
    pool = app.state.db_pool
    async with pool.acquire() as conn:
        row = await conn.fetchrow("""
            INSERT INTO captions.caption_requests (image_path)
            VALUES ($1)
            RETURNING id
        """, save_path)
        request_id = row["id"]

        # 3. Notify
        payload = f'{{"request_id":{request_id},"image_path":"{save_path}"}}'
        await conn.execute("SELECT pg_notify($1, $2)", "caption_request", payload)

    print(f"Uploaded image saved to {save_path}, request_id={request_id}, text_query={text_query}")

    return RedirectResponse(url="/", status_code=303)

@app.get("/search", response_class=HTMLResponse)
async def search_images(request: Request, q: str = ""):
    """
    A route to handle text-based searching for existing images/captions
    For example: naive search by caption_text or image_path
    """
    pool = request.app.state.db_pool
    async with pool.acquire() as conn:
        rows = await conn.fetch("""
            SELECT cr.id as request_id,
                   cr.image_path,
                   resp.caption_text
            FROM captions.caption_requests cr
            LEFT JOIN captions.caption_responses resp
               ON cr.id = resp.caption_request_id
            WHERE
                resp.caption_text ILIKE $1
                OR cr.image_path ILIKE $1
            ORDER BY cr.request_time DESC
        """, f"%{q}%")

    results = []
    for row in rows:
        results.append({
            "request_id": row["request_id"],
            "image_path": row["image_path"],
            "caption_text": row["caption_text"]
        })
    # Reuse the same template, or a different one
    return templates.TemplateResponse("index.html", {"request": request, "results": results, "search_query": q})

if __name__ == "__main__":
    uvicorn.run("main:app", host="127.0.0.1", port=8000, reload=True)
