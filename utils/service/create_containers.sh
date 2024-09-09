#!/bin/bash

# Define paths to the Dockerfiles in alto and bajo
ALTO_CPU_PATH="../Docker_files/alto/cpu"
ALTO_RAM_PATH="../Docker_files/alto/ram"
BAJO_PATH="../Docker_files/bajo"

# Function to generate a random container name using /dev/urandom
generate_random_name() {
    cat /dev/urandom | tr -dc 'a-z0-9' | fold -w 12 | head -n 1
}

# Loop to create 10 containers
for i in {1..10}; do
    # Generate a random name for the container
    container_name=$(generate_random_name)
    
    # Randomly select which type of container to run (high CPU, high RAM, or low consumption)
    random_choice=$((RANDOM % 3))
    
    if [ $random_choice -eq 0 ]; then
        echo "Creating high CPU consumption container: $container_name"
        docker run -d --name "$container_name" high_cpu_container
    elif [ $random_choice -eq 1 ]; then
        echo "Creating high RAM consumption container: $container_name"
        docker run -d --name "$container_name" high_ram_container
    else
        echo "Creating low consumption container: $container_name"
        docker run -d --name "$container_name" --cpus=".01" --memory="6m" low_consumption_container
    fi
done
