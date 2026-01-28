import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import './Settings.css';

interface SettingsProps {
  onClose: () => void;
}

interface ModelSettings {
  provider: string;
  deepseek_key: string | null;
  qwen_key: string | null;
}

export function Settings({ onClose }: SettingsProps) {
  const { t } = useTranslation();
  const [provider, setProvider] = useState('deepseek');
  const [deepseekKey, setDeepseekKey] = useState('');
  const [qwenKey, setQwenKey] = useState('');
  const [loading, setLoading] = useState(false);
  const [testResult, setTestResult] = useState('');
  const [activeTab, setActiveTab] = useState<'model' | 'general'>('model');

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const settings = await invoke<ModelSettings>('get_model_settings');
      setProvider(settings.provider || 'deepseek');
      if (settings.deepseek_key) setDeepseekKey(settings.deepseek_key);
      if (settings.qwen_key) setQwenKey(settings.qwen_key);
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  };

  const handleSave = async () => {
    setLoading(true);
    try {
      await invoke('set_model_settings', {
        provider,
        deepseekKey: deepseekKey || null,
        qwenKey: qwenKey || null,
      });
      setTestResult('✅ 设置已保存');
    } catch (e) {
      setTestResult(`❌ 保存失败: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleTest = async (testProvider: string, apiKey: string) => {
    if (!apiKey) {
      setTestResult('请先输入 API Key');
      return;
    }
    setLoading(true);
    try {
      const res = await invoke<string>('test_model_connection', { provider: testProvider, apiKey });
      setTestResult(`✅ ${res}`);
    } catch (e) {
      setTestResult(`❌ 连接失败: ${e}`);
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
        
        <div className="settings-tabs">
          <button 
            className={`tab ${activeTab === 'model' ? 'active' : ''}`}
            onClick={() => setActiveTab('model')}
          >
            模型配置
          </button>
          <button 
            className={`tab ${activeTab === 'general' ? 'active' : ''}`}
            onClick={() => setActiveTab('general')}
          >
            通用设置
          </button>
        </div>

        <div className="settings-body">
          {activeTab === 'model' && (
            <>
              <div className="form-group">
                <label>选择模型</label>
                <div className="model-selector">
                  <button 
                    className={`model-option ${provider === 'deepseek' ? 'selected' : ''}`}
                    onClick={() => setProvider('deepseek')}
                  >
                    <span className="model-icon">🔮</span>
                    <span className="model-name">DeepSeek</span>
                    <span className="model-desc">推荐</span>
                  </button>
                  <button 
                    className={`model-option ${provider === 'qwen' ? 'selected' : ''}`}
                    onClick={() => setProvider('qwen')}
                  >
                    <span className="model-icon">🌐</span>
                    <span className="model-name">通义千问</span>
                    <span className="model-desc">Qwen</span>
                  </button>
                </div>
              </div>

              <div className="form-group">
                <label>DeepSeek API Key</label>
                <div className="input-row">
                  <input 
                    type="password" 
                    value={deepseekKey} 
                    onChange={(e) => setDeepseekKey(e.target.value)} 
                    placeholder="sk-..."
                  />
                  <button 
                    className="test-btn"
                    onClick={() => handleTest('deepseek', deepseekKey)} 
                    disabled={loading || !deepseekKey}
                  >
                    测试
                  </button>
                </div>
                <a href="https://platform.deepseek.com/api_keys" target="_blank" className="api-link">
                  获取 DeepSeek API Key →
                </a>
              </div>

              <div className="form-group">
                <label>通义千问 API Key</label>
                <div className="input-row">
                  <input 
                    type="password" 
                    value={qwenKey} 
                    onChange={(e) => setQwenKey(e.target.value)} 
                    placeholder="sk-..."
                  />
                  <button 
                    className="test-btn"
                    onClick={() => handleTest('qwen', qwenKey)} 
                    disabled={loading || !qwenKey}
                  >
                    测试
                  </button>
                </div>
                <a href="https://dashscope.console.aliyun.com/apiKey" target="_blank" className="api-link">
                  获取通义千问 API Key →
                </a>
              </div>

              <div className="settings-actions">
                <button className="primary" onClick={handleSave} disabled={loading}>
                  {loading ? '保存中...' : t('common.save')}
                </button>
              </div>
              
              {testResult && <div className="test-result">{testResult}</div>}
            </>
          )}

          {activeTab === 'general' && (
            <div className="coming-soon">
              <p>🚧 更多设置即将推出</p>
              <ul>
                <li>工作区路径配置</li>
                <li>代理最大步数</li>
                <li>命令超时时间</li>
                <li>主题切换</li>
              </ul>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
