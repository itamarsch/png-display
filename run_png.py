import os
import subprocess
import sys


def run_cargo_on_png_files(start_index=0):
    directory = "PngSuite"
    png_files = [f for f in os.listdir(directory) if f.endswith(".png")]
    png_files.sort()  # Sort files to ensure consistent ordering

    for index, filename in enumerate(png_files):
        if index < start_index:
            continue

        filepath = os.path.join(directory, filename)
        print(f"Processing file {index}: {filepath}")
        command = ["cargo", "run", "--", filepath]
        process = subprocess.run(command, capture_output=True, text=True)

        # Output all stdout
        print(process.stdout)

        # Check the exit code
        if process.returncode != 0:
            print(
                f"Error: Process failed for {filename} (index {index}) with exit code {process.returncode}"
            )
            print(process.stderr)
            print(f"Last index processed: {index}")
            return

    print("All files processed successfully.")


if __name__ == "__main__":
    start_index = int(sys.argv[1]) if len(sys.argv) > 1 else 0
    run_cargo_on_png_files(start_index)
