import { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import { TokenBadge, TokenDetails } from './TokenUsage';
import './Stage3.css';

export default function Stage3({ finalResponse }) {
  const [copied, setCopied] = useState(false);

  if (!finalResponse) {
    return null;
  }

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(finalResponse.response);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  return (
    <div className="stage stage3">
      <div className="stage-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '15px' }}>
        <h3 className="stage-title" style={{ margin: 0 }}>Stage 3: Final Council Answer</h3>
        <button 
          className="copy-button" 
          onClick={handleCopy}
          title="Copy as markdown"
          style={{ 
            background: '#4a90e2', 
            color: 'white', 
            border: 'none', 
            padding: '8px 16px', 
            borderRadius: '4px', 
            cursor: 'pointer',
            fontSize: '14px',
            flexShrink: 0
          }}
        >
          {copied ? '✓ Copied!' : '📋 Copy'}
        </button>
      </div>
      <div className="final-response">
        <div className="chairman-label">
          Chairman: {finalResponse.model.split('/')[1] || finalResponse.model}
          <TokenBadge usage={finalResponse.usage} />
        </div>
        <TokenDetails usage={finalResponse.usage} />
        <div className="final-text markdown-content">
          <ReactMarkdown>{finalResponse.response}</ReactMarkdown>
        </div>
      </div>
    </div>
  );
}
