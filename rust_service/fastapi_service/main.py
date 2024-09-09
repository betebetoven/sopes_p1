from fastapi import FastAPI, Request  # Add Request for handling POST requests

app = FastAPI()

@app.get("/")
async def root():
    return {"message": "FastAPI server is running"}

@app.get("/hello")
async def say_hello(name: str):
    return {"message": f"Hello, {name}!"}

@app.post("/log")
async def receive_log(request: Request):
    data = await request.json()
    print("Received data:", data)  # You can process the data here
    return {"message": "Data received successfully"}
