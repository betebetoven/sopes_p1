import time

def consume_ram():
    print("Starting high RAM consumption...")
    large_list = []  # Start with an empty list
    target_allocation = 20000000  # Reduce the total number of elements to allocate

    for i in range(target_allocation):  # Gradually allocate memory
        large_list.append(i)
        if i % 1000000 == 0:
            print(f"Allocated {i} elements so far...")
            time.sleep(1)  # Pause to reduce the speed of allocation and ease system load

    print("Finished allocating memory, keeping process alive...")
    time.sleep(300)  # Keep the program running for a while to observe RAM usage

if __name__ == "__main__":
    consume_ram()
