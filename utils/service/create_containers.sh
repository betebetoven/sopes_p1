#!/bin/bash

# Number of containers to create
NUM_CONTAINERS=10

# Array of image names
IMAGES=("high_ram_image" "high_cpu_image" "low_ram_image" "low_cpu_image")

# Loop to create containers
for i in $(seq 1 $NUM_CONTAINERS)
do
  # Generate a random image
  RANDOM_IMAGE=${IMAGES[$RANDOM % ${#IMAGES[@]}]}
  
  # Generate a random name for the container
  CONTAINER_NAME=$(openssl rand -hex 3)

  # Set limits based on image type
  if [[ "$RANDOM_IMAGE" == "high_ram_image" || "$RANDOM_IMAGE" == "high_cpu_image" ]]; then
      CPU_LIMIT="0.25"  # 25% of a core
      MEMORY_LIMIT="256m"  # 256MB of RAM
  else
      CPU_LIMIT="0.1"  # 10% of a core
      MEMORY_LIMIT="64m"  # 64MB of RAM
  fi
  
  # Run a container with limits
  docker run -d --name "container_$CONTAINER_NAME" --cpus="$CPU_LIMIT" --memory="$MEMORY_LIMIT" $RANDOM_IMAGE
  
  # Print the name and image of the container that was created
  echo "Created container: container_$CONTAINER_NAME from image: $RANDOM_IMAGE"
done
