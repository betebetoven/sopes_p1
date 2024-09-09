from fastapi import FastAPI, Request, WebSocket
from fastapi.responses import HTMLResponse
from typing import Dict
import json

app = FastAPI()

# Global variable to store logs
global_data = {"logs": []}

# HTML content to display on the root endpoint
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
                newData.textContent = event.data;
                dataDiv.appendChild(newData);
            };
        </script>
    </body>
</html>
"""

@app.get("/", response_class=HTMLResponse)
async def get_html():
    # Return the HTML page
    return html_content

@app.get("/hello")
async def say_hello(name: str):
    return {"message": f"Hello, {name}!"}

@app.post("/log")
async def receive_log(request: Request):
    # Receive the data and store it in the global dictionary
    data = await request.json()
    global_data["logs"].append(data)
    print("Received data:", data)  # Process the data here
    return {"message": "Data received successfully"}

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    while True:
        # Continuously send the updated logs to the client
        await websocket.send_text(json.dumps(global_data["logs"]))
        await asyncio.sleep(2)  # Send updates every 2 seconds
