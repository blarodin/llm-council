import { useRef } from 'react';
import './FileUpload.css';

export default function FileUpload({ attachedFiles, onFilesChange, disabled }) {
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

  const formatFileSize = (bytes) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  return (
    <div className="file-upload">
      <input
        ref={fileInputRef}
        type="file"
        onChange={handleFileSelect}
        style={{ display: 'none' }}
        multiple
        accept="image/*,.pdf,.txt,.doc,.docx,.json,.csv"
        disabled={disabled}
      />
      
      <button
        type="button"
        className="attach-file-button"
        onClick={() => fileInputRef.current?.click()}
        disabled={disabled}
        title="Attach files"
      >
        📎
      </button>

      {attachedFiles.length > 0 && (
        <div className="attached-files-list">
          {attachedFiles.map((file, index) => (
            <div key={index} className="attached-file">
              {file.type.startsWith('image/') && (
                <img src={file.data} alt={file.name} className="file-thumbnail" />
              )}
              <div className="file-info">
                <span className="file-name">{file.name}</span>
                <span className="file-size">{formatFileSize(file.size)}</span>
              </div>
              <button
                type="button"
                className="remove-file-button"
                onClick={() => removeFile(index)}
                disabled={disabled}
                title="Remove file"
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
