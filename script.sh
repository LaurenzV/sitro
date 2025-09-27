#!/bin/bash

# Directory containing PDFs
PDF_DIR="pdf"

# Make sure the directory exists
if [ ! -d "$PDF_DIR" ]; then
    echo "Error: Directory '$PDF_DIR' does not exist."
    exit 1
fi

# Loop through each PDF in the directory
for pdf_file in "$PDF_DIR"/*.pdf; do
    if [ -f "$pdf_file" ]; then
        # Get the filename without the extension
        base_name="$(basename "$pdf_file" .pdf)"
        
        # Create cleaned filename
        cleaned_file="$PDF_DIR/${base_name}.pdf"
        
        # Run mutool clean
        echo "Cleaning '$pdf_file' -> '$cleaned_file'"
        mutool clean -gida "$pdf_file" "$cleaned_file"
    fi
done

cargo run --release