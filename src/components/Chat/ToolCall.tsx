import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import './ToolCall.css';

interface ToolCallProps {
  name: string;
  args: string;
  result?: string;
}

export function ToolCallView({ name, args, result }: ToolCallProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="tool-call">
      <div className="tool-header" onClick={() => setExpanded(!expanded)}>
        <span className="tool-icon">🛠️</span>
        <span className="tool-name">{name}</span>
        <span className="tool-status">{result ? '✅' : '⏳'}</span>
        <span className="tool-arrow">{expanded ? '▼' : '▶'}</span>
      </div>
      {expanded && (
        <div className="tool-details">
          <div className="tool-section">
            <div className="tool-label">Args:</div>
            <pre className="tool-code">{args}</pre>
          </div>
          {result && (
            <div className="tool-section">
              <div className="tool-label">{t('chat.toolResult')}:</div>
              <pre className="tool-code result">{result}</pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
