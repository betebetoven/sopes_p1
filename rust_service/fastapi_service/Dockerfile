
FROM python:3.11-slim

# Create app directory (with user)
RUN addgroup --system app && adduser --system --group app
WORKDIR /home/app

# Install dependencies
# First copy requirements.txt and install 
COPY requirements.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# Copy app
COPY . .

# Use an unprivileged user.
#USER app

# Run the application
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
