from fastapi import FastAPI, Request, WebSocket, Depends
from fastapi.staticfiles import StaticFiles
from fastapi.responses import HTMLResponse, FileResponse
from typing import List, Dict
import json
import os
import matplotlib.pyplot as plt

app = FastAPI()
app.mount("/static", StaticFiles(directory="static"), name="static")

LOG_FILE_PATH = "/app/shared/logs.json"  # This should be a shared volume with the host

class LogManager:
    def __init__(self):
        self.logs: List[Dict] = []
        self.websockets: List[WebSocket] = []
        self.load_logs_from_file()

    # Load logs from the shared JSON file when the service starts
    def load_logs_from_file(self):
        if os.path.exists(LOG_FILE_PATH):
            with open(LOG_FILE_PATH, "r") as file:
                self.logs = json.load(file)
        else:
            self.logs = []

    # Save logs to the shared JSON file after adding a new log
    def save_logs_to_file(self):
        with open(LOG_FILE_PATH, "w") as file:
            json.dump(self.logs, file, indent=2)

    async def add_log(self, log: Dict):
        self.logs.append(log)
        self.save_logs_to_file()  # Save the logs to a file
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

# Generate graphs from the logs
@app.get("/generate-graphs")
async def generate_graphs(log_manager: LogManager = Depends(lambda: log_manager)):
    if not log_manager.logs:
        return {"message": "No logs available to generate graphs"}

    # Example data extraction (memory and CPU usage)
    memory_usage = [log["memory_usage"] for log in log_manager.logs if "memory_usage" in log]
    cpu_usage = [log["cpu_usage"] for log in log_manager.logs if "cpu_usage" in log]
    timestamps = [log["timestamp"] for log in log_manager.logs if "timestamp" in log]

    # Plot memory usage
    plt.figure(figsize=(10, 5))
    plt.plot(timestamps, memory_usage, label="Memory Usage", color="blue")
    plt.xlabel("Timestamp")
    plt.ylabel("Memory Usage (%)")
    plt.title("Memory Usage Over Time")
    plt.legend()
    plt.xticks(rotation=45)
    memory_graph_path = "/app/shared/memory_usage.png"  # Save in the shared volume
    plt.savefig(memory_graph_path)
    plt.close()

    # Plot CPU usage
    plt.figure(figsize=(10, 5))
    plt.plot(timestamps, cpu_usage, label="CPU Usage", color="green")
    plt.xlabel("Timestamp")
    plt.ylabel("CPU Usage (%)")
    plt.title("CPU Usage Over Time")
    plt.legend()
    plt.xticks(rotation=45)
    cpu_graph_path = "/app/shared/cpu_usage.png"  # Save in the shared volume
    plt.savefig(cpu_graph_path)
    plt.close()

    return {
        "message": "Graphs generated successfully",
        "memory_graph": memory_graph_path,
        "cpu_graph": cpu_graph_path
    }
