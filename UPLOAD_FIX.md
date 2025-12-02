# Document Upload Fix

## Problem
Document uploading was broken after the migration from Python to Tauri/Rust. The `FileUpload` component existed but was not integrated into the UI.

## Root Cause
During the UI redesign or Tauri migration, a fully-featured `FileUpload.jsx` component was created with:
- File type validation (images, PDFs, text files, markdown, etc.)
- Visual previews with thumbnails for images
- File size display
- Collapsible file list
- Proper remove functionality

However, this component was never imported or used in `ChatInterface.jsx`. Instead, a simpler implementation was left in place with:
- A hidden file input
- Basic file chips display
- No file type validation
- No visual previews

## Changes Made

### 1. ChatInterface.jsx
- **Added import**: Imported the `FileUpload` component
- **Removed duplicate code**: Removed `fileInputRef`, `handleFileSelect`, `handleAttachClick`, and `readFileAsDataURL` functions (all now handled by FileUpload component)
- **Integrated FileUpload**: Replaced the hidden file input + attach button + chips display with the FileUpload component
- **Cleaner layout**: Simplified the input form structure

### 2. ChatInterface.css
- **Added gap**: Added `gap: 12px` to `.input-form` for proper spacing between FileUpload and textarea

## Features Now Working

### File Upload UI
✅ Visual file input with "Choose Files" button  
✅ Accepts: images, PDFs, text files, doc/docx, JSON, CSV, markdown  
✅ Multiple file selection  
✅ File preview with thumbnails for images  
✅ File type icons for non-images (📄 for PDFs, 📝 for markdown, 📎 for others)  
✅ File size display  
✅ Collapsible file list  
✅ Remove files individually  
✅ Disabled state during message sending  

### Backend Processing (Already Working)
✅ Base64 file encoding  
✅ Text extraction from text-based files  
✅ Image support for vision models  
✅ Files included in conversation storage  
✅ Files passed to all LLM council stages  

## How to Test

1. Start the dev server:
   ```bash
   npm run dev
   ```

2. Create a new conversation

3. Click "Choose Files" button in the input area

4. Select one or more files:
   - Try `test_upload.md` in the project root
   - Try an image file
   - Try a text file

5. Files should appear with previews/icons

6. Type a message like "What's in this file?" and submit

7. The LLM Council should reference the file content in their responses

## Architecture

```
┌─────────────────────────────────────────┐
│         ChatInterface.jsx               │
│  ┌───────────────────────────────────┐  │
│  │      FileUpload.jsx               │  │
│  │  - File selection                 │  │
│  │  - Preview/thumbnails             │  │
│  │  - Remove files                   │  │
│  │  - State: attachedFiles           │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │      Textarea                     │  │
│  │  - Message input                  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
          │
          ▼ onSendMessage(content, files)
┌─────────────────────────────────────────┐
│         api.sendMessageStream()         │
│  - Sends content + files array          │
│  - Files: [{name, type, size, data}]    │
└─────────────────────────────────────────┘
          │
          ▼ POST /api/conversations/:id/message/stream
┌─────────────────────────────────────────┐
│      Rust Backend (api.rs)              │
│  - Receives SendMessageRequest          │
│  - Extracts files array                 │
└─────────────────────────────────────────┘
          │
          ▼ extract_text_from_files()
┌─────────────────────────────────────────┐
│      Council Processing (council.rs)    │
│  - Stage 1: Appends file content to     │
│    user query for text files            │
│  - Stage 1: Adds image_url for images   │
│  - Stage 2: Includes files in rankings  │
│  - Stage 3: Includes files in synthesis │
└─────────────────────────────────────────┘
```

## Supported File Types

### Text Extraction (appended to prompt)
- `.txt` - Plain text
- `.md` - Markdown
- `.json` - JSON
- `.csv` - CSV
- Other text-based formats

### Image Vision (sent as image_url to vision models)
- `.jpg`, `.jpeg`
- `.png`
- `.gif`
- `.webp`

### Binary (currently skipped from text extraction)
- `.pdf` - PDF (future: OCR integration)
- `.doc`, `.docx` - Word documents (future: conversion)

## Future Enhancements

1. **PDF text extraction**: Use a PDF parsing library to extract text from PDFs
2. **OCR for images**: Extract text from images containing text
3. **Drag & drop**: Add drag-and-drop file upload support
4. **File size limits**: Add client-side validation for max file size
5. **Progress indicators**: Show upload progress for large files
6. **Document preview**: Preview text files inline before sending
