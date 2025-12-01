import { useState } from 'react';
import './TokenUsage.css';

function formatNumber(num) {
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'K';
  }
  return num.toString();
}

export function TokenBadge({ usage }) {
  if (!usage || !usage.total_tokens) {
    return null;
  }

  return (
    <span className="token-badge">
      {formatNumber(usage.total_tokens)} tokens
    </span>
  );
}

export function TokenDetails({ usage }) {
  if (!usage || !usage.total_tokens) {
    return null;
  }

  return (
    <div className="token-details">
      <span className="token-item">
        <span className="token-label">Prompt:</span> {formatNumber(usage.prompt_tokens)}
      </span>
      <span className="token-item">
        <span className="token-label">Completion:</span> {formatNumber(usage.completion_tokens)}
      </span>
      <span className="token-item token-total">
        <span className="token-label">Total:</span> {formatNumber(usage.total_tokens)}
      </span>
    </div>
  );
}

export function TokenSummary({ usageSummary }) {
  const [isExpanded, setIsExpanded] = useState(false);

  if (!usageSummary) {
    return null;
  }

  const { stage1_total, stage2_total, stage3_total, grand_total, by_model } = usageSummary;

  return (
    <div className="token-summary">
      <div className="token-summary-header" onClick={() => setIsExpanded(!isExpanded)}>
        <h4>Token Usage Summary</h4>
        <span className="token-summary-toggle">{isExpanded ? '▼' : '▶'}</span>
      </div>
      
      {isExpanded && (
        <>
      <div className="summary-section">
        <h5>By Stage</h5>
        <div className="summary-grid">
          <div className="summary-item">
            <span className="summary-label">Stage 1 (Responses):</span>
            <span className="summary-value">{formatNumber(stage1_total.total_tokens)}</span>
          </div>
          <div className="summary-item">
            <span className="summary-label">Stage 2 (Rankings):</span>
            <span className="summary-value">{formatNumber(stage2_total.total_tokens)}</span>
          </div>
          <div className="summary-item">
            <span className="summary-label">Stage 3 (Synthesis):</span>
            <span className="summary-value">{formatNumber(stage3_total.total_tokens)}</span>
          </div>
          <div className="summary-item summary-total">
            <span className="summary-label">Grand Total:</span>
            <span className="summary-value">{formatNumber(grand_total.total_tokens)}</span>
          </div>
        </div>
      </div>

      {Object.keys(by_model).length > 0 && (
        <div className="summary-section">
          <h5>By Model</h5>
          <div className="summary-grid">
            {Object.entries(by_model)
              .sort((a, b) => b[1].total_tokens - a[1].total_tokens)
              .map(([model, usage]) => (
                <div key={model} className="summary-item">
                  <span className="summary-label">{model.split('/')[1] || model}:</span>
                  <span className="summary-value">{formatNumber(usage.total_tokens)}</span>
                  <span className="summary-breakdown">
                    ({formatNumber(usage.prompt_tokens)} prompt + {formatNumber(usage.completion_tokens)} completion)
                  </span>
                </div>
              ))}
          </div>
        </div>
      )}
        </>
      )}
    </div>
  );
}
