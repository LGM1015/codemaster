import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import './ChatInput.css';

interface ChatInputProps {
  onSend: (message: string) => void;
  disabled?: boolean;
}

export function ChatInput({ onSend, disabled }: ChatInputProps) {
  const { t } = useTranslation();
  const [input, setInput] = useState('');

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleSend = () => {
    if (input.trim() && !disabled) {
      onSend(input);
      setInput('');
    }
  };

  return (
    <div className="chat-input-container">
      <textarea
        className="chat-textarea"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={t('chat.placeholder')}
        disabled={disabled}
        rows={3}
      />
      <button className="chat-send-btn" onClick={handleSend} disabled={disabled || !input.trim()}>
        {t('chat.send')}
      </button>
    </div>
  );
}
