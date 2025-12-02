import { useState, useEffect, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import Stage1 from './Stage1';
import Stage2 from './Stage2';
import Stage3 from './Stage3';
import { TokenSummary } from './TokenUsage';
import FileUpload from './FileUpload';
import './ChatInterface.css';

// Helper component to display text file content
function TextFilePreview({ file }) {
  const [content, setContent] = useState('');
  const [isExpanded, setIsExpanded] = useState(false);

  useEffect(() => {
    // Extract text from base64 data URL
    try {
      const base64Data = file.data.split('base64,')[1];
      const decoded = atob(base64Data);
      setContent(decoded);
    } catch (e) {
      setContent('(Unable to display file content)');
    }
  }, [file.data]);

  const displayContent = content.length > 300 && !isExpanded 
    ? content.substring(0, 300) + '...' 
    : content;

  return (
    <div className="text-file-preview">
      <pre className="text-file-content">{displayContent}</pre>
      {content.length > 300 && (
        <button 
          className="expand-button" 
          onClick={() => setIsExpanded(!isExpanded)}
        >
          {isExpanded ? 'Show less' : 'Show more'}
        </button>
      )}
    </div>
  );
}

export default function ChatInterface({
  conversation,
  onSendMessage,
  isLoading,
  isSidebarCollapsed,
  onToggleSidebar,
}) {
  const [input, setInput] = useState('');
  const [attachedFiles, setAttachedFiles] = useState([]);
  const [welcomeInput, setWelcomeInput] = useState('');
  const messagesEndRef = useRef(null);
  const textareaRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [conversation]);


  const handleSubmit = (e) => {
    e.preventDefault();
    if ((input.trim() || attachedFiles.length > 0) && !isLoading) {
      onSendMessage(input, attachedFiles);
      setInput('');
      setAttachedFiles([]);
      // Reset textarea height
      if (textareaRef.current) {
        textareaRef.current.style.height = 'auto';
      }
    }
  };

  const handleWelcomeSubmit = (e) => {
    e.preventDefault();
    if (welcomeInput.trim() && !isLoading) {
      onSendMessage(welcomeInput, []);
      setWelcomeInput('');
    }
  };

  const handleKeyDown = (e) => {
    // Submit on Enter (without Shift)
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  const handleInputChange = (e) => {
    setInput(e.target.value);
    // Auto-resize textarea
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = Math.min(textarea.scrollHeight, 200) + 'px';
    }
  };

  const handleWelcomeKeyDown = (e) => {
    // Submit on Enter
    if (e.key === 'Enter') {
      e.preventDefault();
      handleWelcomeSubmit(e);
    }
  };


  const showEmptyState = !conversation || conversation.messages.length === 0;

  if (!conversation) {
    return (
      <div className="chat-interface">
        <div className="chat-topbar">
          <button
            className="sidebar-toggle-btn"
            onClick={onToggleSidebar}
            title={isSidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          >
            ☰
          </button>
          <div className="topbar-title">LLM Council</div>
        </div>
        <div className="welcome-screen">
          <h1 className="welcome-title">Hi, how are you?</h1>
          <p className="welcome-subtitle">How can I help you today?</p>
        </div>
      </div>
    );
  }

  return (
    <div className="chat-interface">
      <div className="chat-topbar">
        <button
          className="sidebar-toggle-btn"
          onClick={onToggleSidebar}
          title={isSidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          ☰
        </button>
        <div className="topbar-title">LLM Council</div>
      </div>
      <div className="messages-container">
        {conversation.messages.length === 0 ? (
          <div className="welcome-screen-with-conversation">
            <h1 className="welcome-title">Hi, how are you?</h1>
            <p className="welcome-subtitle">How can I help you today?</p>
          </div>
        ) : (
          conversation.messages.map((msg, index) => (
            <div key={index} className="message-group">
              {msg.role === 'user' ? (
                <div className="user-message">
                  <div className="message-label">You</div>
                  <div className="message-content">
                    {msg.files && msg.files.length > 0 && (
                      <div className="attached-files-list">
                        <div className="files-list-header">📎 Attached files:</div>
                        <ul className="files-list">
                          {msg.files.map((file, idx) => (
                            <li key={idx} className="file-list-item">
                              {file.name}
                            </li>
                          ))}
                        </ul>
                      </div>
                    )}
                    <div className="markdown-content">
                      <ReactMarkdown>{msg.content}</ReactMarkdown>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="assistant-message">
                  <div className="message-label">LLM Council</div>

                  {/* Stage 1 */}
                  {msg.loading?.stage1 && (
                    <div className="stage-loading">
                      <div className="spinner"></div>
                      <span>Running Stage 1: Collecting individual responses...</span>
                    </div>
                  )}
                  {msg.stage1 && <Stage1 responses={msg.stage1} />}

                  {/* Stage 2 */}
                  {msg.loading?.stage2 && (
                    <div className="stage-loading">
                      <div className="spinner"></div>
                      <span>Running Stage 2: Peer rankings...</span>
                    </div>
                  )}
                  {msg.stage2 && (
                    <Stage2
                      rankings={msg.stage2}
                      labelToModel={msg.metadata?.label_to_model}
                      aggregateRankings={msg.metadata?.aggregate_rankings}
                    />
                  )}

                  {/* Stage 3 */}
                  {msg.loading?.stage3 && (
                    <div className="stage-loading">
                      <div className="spinner"></div>
                      <span>Running Stage 3: Final synthesis...</span>
                    </div>
                  )}
                  {msg.stage3 && <Stage3 finalResponse={msg.stage3} />}

                  {/* Token Usage Summary */}
                  {msg.metadata?.usage_summary && (
                    <TokenSummary usageSummary={msg.metadata.usage_summary} />
                  )}
                </div>
              )}
            </div>
          ))
        )}

        {isLoading && (
          <div className="loading-indicator">
            <div className="spinner"></div>
            <span>Consulting the council...</span>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <form className="input-form" onSubmit={handleSubmit}>
        {attachedFiles.length > 0 && (
          <FileUpload
            attachedFiles={attachedFiles}
            onFilesChange={setAttachedFiles}
            disabled={isLoading}
            showOnlyChips={true}
          />
        )}
        <div className="input-area">
          <div className="input-wrapper">
            <textarea
              ref={textareaRef}
              className="message-input"
              placeholder="Type your message here..."
              value={input}
              onChange={handleInputChange}
              onKeyDown={handleKeyDown}
              disabled={isLoading}
              rows={1}
            />
          </div>
          <div className="input-actions">
            <FileUpload
              attachedFiles={attachedFiles}
              onFilesChange={setAttachedFiles}
              disabled={isLoading}
            />
          </div>
        </div>
      </form>
    </div>
  );
}
