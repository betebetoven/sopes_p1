import time

def consume_cpu():
    print("Starting high CPU consumption...")
    while True:
        # Increase CPU work by adding a nested loop with larger ranges
        for i in range(10000):
            for j in range(10000):
                for k in range(10000):
                    for l in range(10000):
                        x = i * j * k * l

if __name__ == "__main__":
    consume_cpu()
