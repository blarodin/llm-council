import { useRef, useState } from 'react';
import './FileUpload.css';

export default function FileUpload({ attachedFiles, onFilesChange, disabled }) {
  const fileInputRef = useRef(null);
  const [isExpanded, setIsExpanded] = useState(false);

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
        className="file-input-field"
        multiple
        accept="image/*,.pdf,.txt,.doc,.docx,.json,.csv,.md"
        disabled={disabled}
      />

      {attachedFiles.length > 0 && (
        <div className="attached-files-section">
          <div className="attached-files-header" onClick={() => setIsExpanded(!isExpanded)}>
            <span className="file-count">
              {attachedFiles.length} {attachedFiles.length === 1 ? 'file' : 'files'} attached
            </span>
            <button
              type="button"
              className="toggle-button"
              aria-label={isExpanded ? 'Collapse' : 'Expand'}
            >
              {isExpanded ? '▼' : '▶'}
            </button>
          </div>
          {isExpanded && (
            <div className="attached-files-list">
          {attachedFiles.map((file, index) => (
            <div key={index} className="attached-file">
              <button
                type="button"
                className="remove-file-button"
                onClick={() => removeFile(index)}
                disabled={disabled}
                title="Remove file"
              >
                ×
              </button>
              <div className="attached-file-preview">
                {file.type.startsWith('image/') ? (
                  <img src={file.data} alt={file.name} className="file-thumbnail" />
                ) : (
                  <div className="file-thumbnail non-image">
                    {file.type.includes('pdf') ? '📄' : 
                     file.name.endsWith('.md') ? '📝' : '📎'}
                  </div>
                )}
                <div className="file-info">
                  <span className="file-name" title={file.name}>{file.name}</span>
                  <span className="file-size">{formatFileSize(file.size)}</span>
                </div>
              </div>
            </div>
          ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
