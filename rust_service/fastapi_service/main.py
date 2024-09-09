from fastapi import FastAPI, Request, WebSocket, Depends
from fastapi.staticfiles import StaticFiles
from fastapi.responses import HTMLResponse, FileResponse
from typing import List, Dict
import json
import asyncio

app = FastAPI()
app.mount("/static", StaticFiles(directory="static"), name="static")

class LogManager:
    def __init__(self):
        self.logs: List[Dict] = []
        self.websockets: List[WebSocket] = []

    async def add_log(self, log: Dict):
        self.logs.append(log)
        await self.broadcast(log)

    async def broadcast(self, message: Dict):
        for websocket in self.websockets:
            await websocket.send_text(json.dumps(message))

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.websockets.append(websocket)
        await websocket.send_text(json.dumps({"type": "history", "logs": self.logs}))

    def disconnect(self, websocket: WebSocket):
        self.websockets.remove(websocket)

log_manager = LogManager()

html_content = """
<!DOCTYPE html>
<html>
    <head>
        <title>WebSocket Live Data</title>
    </head>
    <body>
        <h1>Live Data Stream</h1>
        <div id="data"></div>
        <script>
            var ws = new WebSocket("ws://localhost:8000/ws");

            ws.onmessage = function(event) {
                var dataDiv = document.getElementById("data");
                var newData = document.createElement("p");
                newData.textContent = JSON.stringify(JSON.parse(event.data), null, 2);
                dataDiv.appendChild(newData);
            };
        </script>
    </body>
</html>
"""

@app.get("/", response_class=HTMLResponse)
async def get_html():
    return FileResponse("static/dashboard.html")

@app.get("/hello")
async def say_hello(name: str):
    return {"message": f"Hello, {name}!"}

@app.post("/log")
async def receive_log(request: Request, log_manager: LogManager = Depends(lambda: log_manager)):
    data = await request.json()
    await log_manager.add_log(data)
    return {"message": "Data received successfully"}

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket, log_manager: LogManager = Depends(lambda: log_manager)):
    await log_manager.connect(websocket)
    try:
        while True:
            await websocket.receive_text()  # Keep the connection alive
    except:
        log_manager.disconnect(websocket)