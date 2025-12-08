import { useState, useEffect, useRef } from 'react';
import './Sidebar.css';

export default function Sidebar({
  conversations,
  currentConversationId,
  onSelectConversation,
  onNewConversation,
  onDeleteConversation,
  onRenameConversation,
  onOpenSettings,
  isCollapsed,
  onToggleCollapse,
  width,
  onResize,
}) {
  const [editingId, setEditingId] = useState(null);
  const [editingTitle, setEditingTitle] = useState('');
  const [deletingId, setDeletingId] = useState(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [isResizing, setIsResizing] = useState(false);
  const inputRef = useRef(null);
  const isSavingRef = useRef(false);
  const sidebarRef = useRef(null);

  const handleStartRename = (conv, e) => {
    e.stopPropagation();
    setEditingId(conv.id);
    setEditingTitle(conv.title || 'New Conversation');
  };

  const handleCancelRename = () => {
    isSavingRef.current = false;
    setEditingId(null);
    setEditingTitle('');
  };

  const handleSaveRename = (id) => {
    if (isSavingRef.current) return;
    isSavingRef.current = true;
    
    if (editingTitle.trim()) {
      onRenameConversation(id, editingTitle.trim());
    }
    setEditingId(null);
    setEditingTitle('');
    
    // Reset flag after a small delay
    setTimeout(() => {
      isSavingRef.current = false;
    }, 100);
  };

  const handleRenameKeyDown = (e, id) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSaveRename(id);
    } else if (e.key === 'Escape') {
      handleCancelRename();
    }
  };

  // Focus input when editing starts
  useEffect(() => {
    if (editingId && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editingId]);

  // Handle resize
  useEffect(() => {
    const handleMouseMove = (e) => {
      if (!isResizing) return;
      e.preventDefault();
      const newWidth = e.clientX;
      if (newWidth >= 200 && newWidth <= 600) {
        onResize(newWidth);
      }
    };

    const handleMouseUp = (e) => {
      if (isResizing) {
        e.preventDefault();
      }
      setIsResizing(false);
    };

    const handleSelectStart = (e) => {
      if (isResizing) {
        e.preventDefault();
      }
    };

    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      document.addEventListener('selectstart', handleSelectStart);
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.removeEventListener('selectstart', handleSelectStart);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };
  }, [isResizing, onResize]);

  const filteredConversations = conversations.filter(conv => {
    const title = conv.title || 'New Conversation';
    return title.toLowerCase().includes(searchQuery.toLowerCase());
  });

  return (
    <div 
      ref={sidebarRef}
      className={`sidebar ${isCollapsed ? 'collapsed' : ''} ${isResizing ? 'resizing' : ''}`}
      style={{ width: isCollapsed ? '50px' : `${width}px` }}
    >
      {!isCollapsed && (
        <div
          className="resize-handle"
          onMouseDown={(e) => {
            e.preventDefault();
            setIsResizing(true);
          }}
        />
      )}
      <>
          <div className="sidebar-header">
            <div className="search-container">
              <span className="search-icon">🔍</span>
              <input 
                type="text" 
                className="search-input" 
                placeholder="Search"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
            </div>
            
            <button className="new-chat-btn" onClick={onNewConversation}>
              <span className="btn-icon">➕</span>
              <span>New Chat</span>
            </button>
          </div>

          <div className="conversation-list">
            {filteredConversations.length > 0 && (
              <div className="threads-container">
                {filteredConversations.map((conv) => (
                  <div
                    key={conv.id}
                    className={`conversation-item ${
                      conv.id === currentConversationId ? 'active' : ''
                    } ${deletingId === conv.id ? 'deleting' : ''}`}
                    onClick={() => onSelectConversation(conv.id)}
                  >
                    <div className="conversation-content">
                      {editingId === conv.id ? (
                        <input
                          ref={inputRef}
                          type="text"
                          className="conversation-title-input"
                          value={editingTitle}
                          onChange={(e) => setEditingTitle(e.target.value)}
                          onKeyDown={(e) => handleRenameKeyDown(e, conv.id)}
                          onBlur={() => handleSaveRename(conv.id)}
                          onClick={(e) => e.stopPropagation()}
                        />
                      ) : (
                        <div className="conversation-title">
                          {conv.title || 'New Conversation'}
                        </div>
                      )}
                    </div>
                    <div className="conversation-actions">
                      {editingId !== conv.id && (
                        <button
                          className="rename-btn"
                          onClick={(e) => handleStartRename(conv, e)}
                          title="Rename conversation"
                        >
                          ✎
                        </button>
                      )}
                      <button
                        className={`delete-btn ${deletingId === conv.id ? 'confirming' : ''}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          if (deletingId === conv.id) {
                            onDeleteConversation(conv.id);
                            setDeletingId(null);
                          } else {
                            setDeletingId(conv.id);
                          }
                        }}
                        onMouseLeave={() => {
                          if (deletingId === conv.id) {
                            setDeletingId(null);
                          }
                        }}
                        title={deletingId === conv.id ? 'Click again to confirm deletion' : 'Delete conversation'}
                      >
                        {deletingId === conv.id ? 'Delete?' : '×'}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
          
          <div className="sidebar-footer">
            <button className="settings-btn" onClick={onOpenSettings}>
              <span className="btn-icon">⚙️</span>
              <span>Settings</span>
            </button>
          </div>
      </>
    </div>
  );
}
