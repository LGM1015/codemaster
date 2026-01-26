import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import './Settings.css';

interface SettingsProps {
  onClose: () => void;
}

export function Settings({ onClose }: SettingsProps) {
  const { t } = useTranslation();
  const [apiKey, setApiKey] = useState('');
  const [loading, setLoading] = useState(false);
  const [testResult, setTestResult] = useState('');

  useEffect(() => {
    invoke('get_api_key').then((key) => setApiKey(key as string)).catch(() => {});
  }, []);

  const handleSave = async () => {
    setLoading(true);
    try {
      await invoke('set_api_key', { apiKey });
      setTestResult('Saved successfully');
    } catch (e) {
      setTestResult(`Error: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleTest = async () => {
    setLoading(true);
    try {
      // If user hasn't saved yet, use current input
      const res = await invoke('test_connection', { apiKey });
      setTestResult(res as string);
    } catch (e) {
      setTestResult(`Connection Failed: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="settings-overlay">
      <div className="settings-modal">
        <header className="settings-header">
          <h2>{t('settings.title')}</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </header>
        <div className="settings-body">
          <div className="form-group">
            <label>{t('settings.apiKey')}</label>
            <input 
              type="password" 
              value={apiKey} 
              onChange={(e) => setApiKey(e.target.value)} 
              placeholder={t('settings.apiKeyPlaceholder')}
            />
          </div>
          <div className="settings-actions">
            <button onClick={handleTest} disabled={loading || !apiKey}>{t('settings.testConnection')}</button>
            <button className="primary" onClick={handleSave} disabled={loading || !apiKey}>{t('common.save')}</button>
          </div>
          {testResult && <div className="test-result">{testResult}</div>}
        </div>
      </div>
    </div>
  );
}
