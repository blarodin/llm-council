import { useState, useEffect } from 'react';
import './Settings.css';

export default function Settings({ onClose }) {
  const [apiKey, setApiKey] = useState('');
  const [savedApiKey, setSavedApiKey] = useState('');
  const [councilModels, setCouncilModels] = useState([]);
  const [chairmanModel, setChairmanModel] = useState('');
  const [savedModels, setSavedModels] = useState({ council: [], chairman: '' });
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState(null);
  const [showKey, setShowKey] = useState(false);
  const [newModelInput, setNewModelInput] = useState('');

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      // Load API key
      const keyResponse = await fetch('http://localhost:8001/api/settings/openrouter-key');
      if (keyResponse.ok) {
        const keyData = await keyResponse.json();
        setSavedApiKey(keyData.api_key || '');
        setApiKey(keyData.api_key || '');
      }

      // Load models
      const modelsResponse = await fetch('http://localhost:8001/api/settings/models');
      if (modelsResponse.ok) {
        const modelsData = await modelsResponse.json();
        setCouncilModels(modelsData.council_models || []);
        setChairmanModel(modelsData.chairman_model || '');
        setSavedModels({
          council: modelsData.council_models || [],
          chairman: modelsData.chairman_model || '',
        });
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  };

  const handleSave = async () => {
    setIsLoading(true);
    setMessage(null);
    
    try {
      const response = await fetch('http://localhost:8001/api/settings/openrouter-key', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ api_key: apiKey }),
      });

      if (response.ok) {
        setSavedApiKey(apiKey);
        setMessage({ type: 'success', text: 'API key saved successfully!' });
        setTimeout(() => setMessage(null), 3000);
      } else {
        const error = await response.text();
        setMessage({ type: 'error', text: `Failed to save: ${error}` });
      }
    } catch (error) {
      setMessage({ type: 'error', text: 'Failed to save API key' });
      console.error('Failed to save API key:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClear = async () => {
    setIsLoading(true);
    setMessage(null);
    
    try {
      const response = await fetch('http://localhost:8001/api/settings/openrouter-key', {
        method: 'DELETE',
      });

      if (response.ok) {
        setApiKey('');
        setSavedApiKey('');
        setMessage({ type: 'success', text: 'API key cleared successfully!' });
        setTimeout(() => setMessage(null), 3000);
      } else {
        const error = await response.text();
        setMessage({ type: 'error', text: `Failed to clear: ${error}` });
      }
    } catch (error) {
      setMessage({ type: 'error', text: 'Failed to clear API key' });
      console.error('Failed to clear API key:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSaveModels = async () => {
    setIsLoading(true);
    setMessage(null);
    
    try {
      const response = await fetch('http://localhost:8001/api/settings/models', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          council_models: councilModels, 
          chairman_model: chairmanModel 
        }),
      });

      if (response.ok) {
        setSavedModels({ council: councilModels, chairman: chairmanModel });
        setMessage({ type: 'success', text: 'Models configuration saved successfully!' });
        setTimeout(() => setMessage(null), 3000);
      } else {
        const error = await response.text();
        setMessage({ type: 'error', text: `Failed to save: ${error}` });
      }
    } catch (error) {
      setMessage({ type: 'error', text: 'Failed to save models configuration' });
      console.error('Failed to save models:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddCouncilModel = () => {
    if (newModelInput.trim() && !councilModels.includes(newModelInput.trim())) {
      setCouncilModels([...councilModels, newModelInput.trim()]);
      setNewModelInput('');
    }
  };

  const handleRemoveCouncilModel = (index) => {
    setCouncilModels(councilModels.filter((_, i) => i !== index));
  };

  const hasChanges = apiKey !== savedApiKey;
  const hasModelChanges = 
    JSON.stringify(councilModels) !== JSON.stringify(savedModels.council) ||
    chairmanModel !== savedModels.chairman;

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-modal" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        
        <div className="settings-content">
          <div className="settings-section">
            <h3>OpenRouter API Configuration</h3>
            <p className="settings-description">
              Enter your OpenRouter API key to enable LLM Council. You can get an API key from{' '}
              <a href="https://openrouter.ai/keys" target="_blank" rel="noopener noreferrer">
                openrouter.ai/keys
              </a>
            </p>
            
            <div className="form-group">
              <label htmlFor="api-key">API Key</label>
              <div className="input-with-toggle">
                <input
                  id="api-key"
                  type={showKey ? 'text' : 'password'}
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="sk-or-v1-..."
                  disabled={isLoading}
                  autoComplete="off"
                />
                <button
                  type="button"
                  className="toggle-visibility-btn"
                  onClick={() => setShowKey(!showKey)}
                  title={showKey ? 'Hide API key' : 'Show API key'}
                >
                  {showKey ? '👁️' : '👁️‍🗨️'}
                </button>
              </div>
            </div>

            {message && (
              <div className={`message ${message.type}`}>
                {message.text}
              </div>
            )}

            <div className="settings-actions">
              <button
                className="btn-primary"
                onClick={handleSave}
                disabled={isLoading || !apiKey.trim() || !hasChanges}
              >
                {isLoading ? 'Saving...' : 'Save API Key'}
              </button>
              <button
                className="btn-secondary"
                onClick={handleClear}
                disabled={isLoading || !savedApiKey}
              >
                Clear API Key
              </button>
            </div>
          </div>

          <div className="settings-section">
            <h3>Model Configuration</h3>
            <p className="settings-description">
              Configure which models participate in the council and which model acts as the chairman.
            </p>
            
            <div className="form-group">
              <label htmlFor="chairman-model">Chairman Model</label>
              <input
                id="chairman-model"
                type="text"
                value={chairmanModel}
                onChange={(e) => setChairmanModel(e.target.value)}
                placeholder="e.g., google/gemini-3-pro-preview"
                disabled={isLoading}
              />
              <small className="input-hint">The model that synthesizes final responses</small>
            </div>

            <div className="form-group">
              <label>Council Models ({councilModels.length})</label>
              <div className="models-list">
                {councilModels.map((model, index) => (
                  <div key={index} className="model-item">
                    <span className="model-name">{model}</span>
                    <button
                      className="remove-model-btn"
                      onClick={() => handleRemoveCouncilModel(index)}
                      disabled={isLoading}
                      title="Remove model"
                    >
                      ×
                    </button>
                  </div>
                ))}
              </div>
              <div className="add-model-input">
                <input
                  type="text"
                  value={newModelInput}
                  onChange={(e) => setNewModelInput(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleAddCouncilModel()}
                  placeholder="e.g., openai/gpt-4"
                  disabled={isLoading}
                />
                <button
                  className="add-model-btn"
                  onClick={handleAddCouncilModel}
                  disabled={isLoading || !newModelInput.trim()}
                >
                  Add Model
                </button>
              </div>
              <small className="input-hint">Models that provide responses and rankings</small>
            </div>

            <div className="settings-actions">
              <button
                className="btn-primary"
                onClick={handleSaveModels}
                disabled={isLoading || !chairmanModel || councilModels.length === 0 || !hasModelChanges}
              >
                {isLoading ? 'Saving...' : 'Save Models'}
              </button>
            </div>
          </div>

          <div className="settings-section">
            <h3>About</h3>
            <p className="settings-description">
              LLM Council uses multiple AI models to provide diverse perspectives and synthesized responses.
              The API key is stored locally and only used to communicate with OpenRouter.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
