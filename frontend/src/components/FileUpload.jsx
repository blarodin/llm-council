import { useRef } from 'react';
import './FileUpload.css';

export default function FileUpload({ attachedFiles, onFilesChange, disabled, showOnlyButton = false, showOnlyChips = false }) {
  const fileInputRef = useRef(null);

  const handleFileSelect = async (e) => {
    const files = Array.from(e.target.files);
    if (files.length === 0) return;

    const processedFiles = await Promise.all(
      files.map(async (file) => {
        const data = await readFileAsDataURL(file);
        return {
          name: file.name,
          type: file.type,
          size: file.size,
          data: data,
        };
      })
    );

    onFilesChange([...attachedFiles, ...processedFiles]);
    // Reset the input so the same file can be selected again if needed
    e.target.value = '';
  };

  const readFileAsDataURL = (file) => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result);
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  };

  const removeFile = (index) => {
    const newFiles = attachedFiles.filter((_, idx) => idx !== index);
    onFilesChange(newFiles);
  };

  const handleAttachClick = () => {
    fileInputRef.current?.click();
  };

  if (showOnlyChips) {
    return (
      <div className="attached-files-chips">
        {attachedFiles.map((file, index) => (
          <div key={index} className="file-chip">
            <span className="file-chip-icon">📄</span>
            <span className="file-chip-name" title={file.name}>{file.name}</span>
            <button
              type="button"
              className="file-chip-remove"
              onClick={() => removeFile(index)}
              disabled={disabled}
              title="Remove file"
            >
              ×
            </button>
          </div>
        ))}
      </div>
    );
  }

  return (
    <>
      <input
        ref={fileInputRef}
        type="file"
        onChange={handleFileSelect}
        className="file-input-hidden"
        multiple
        accept="image/*,.pdf,.txt,.doc,.docx,.json,.csv,.md"
        disabled={disabled}
      />

      <button
        type="button"
        className="attach-file-button"
        onClick={handleAttachClick}
        disabled={disabled}
        title="Attach files"
      >
        +
      </button>
    </>
  );
}
