#!/usr/bin/env bash

# Path to the list of files and directories
list_file="package.list"

# Output filenames for the archives
zip_filename="target/source.zip"
tar_filename="target/source.tar.gz"

# Check if the package.list file exists
if [ ! -f "$list_file" ]; then
    echo "Error: $list_file not found!"
    exit 1
fi

mapfile -t files < "$list_file"

# Check if the list is empty
if [ ${#files[@]} -eq 0 ]; then
    echo "Error: $list_file is empty or contains no valid paths!"
    exit 1
fi

# Create the target directory if it doesn't exist
if [ ! -d "target" ]; then
    echo "Creating target directory."
    mkdir -p target
fi

# Create a .zip archive
echo "Creating ZIP archive: $zip_filename"
zip -r "$zip_filename" "${files[@]}"

# Check if zip command was successful
if [ $? -eq 0 ]; then
    echo "ZIP archive created successfully."
else
    echo "Error creating ZIP archive."
    exit 1
fi

# Create a .tar.gz archive
echo "Creating TAR.GZ archive: $tar_filename"
tar -czf "$tar_filename" "${files[@]}"

# Check if tar command was successful
if [ $? -eq 0 ]; then
    echo "TAR.GZ archive created successfully."
else
    echo "Error creating TAR.GZ archive."
    exit 1
fi

echo "Archives created successfully: $zip_filename and $tar_filename."
