import { useState, ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import './Layout.css';

interface LayoutProps {
  sidebar: ReactNode;
  content: ReactNode;
  terminal: ReactNode;
  onSettingsClick: () => void;
}

export function Layout({ sidebar, content, terminal, onSettingsClick }: LayoutProps) {
  const { i18n } = useTranslation();
  const [showTerminal, setShowTerminal] = useState(true);

  const toggleLanguage = () => {
    i18n.changeLanguage(i18n.language === 'en' ? 'zh' : 'en');
  };

  return (
    <div className="layout">
      <div className="layout-body">
        <aside className="layout-sidebar">
          {sidebar}
        </aside>
        <main className="layout-main">
          <header className="layout-header">
             <div className="title">CodeMaster</div>
             <div className="actions">
               <button className="icon-btn" onClick={toggleLanguage} title="Switch Language">
                 {i18n.language === 'en' ? '中' : 'En'}
               </button>
               <button className="icon-btn" onClick={() => setShowTerminal(!showTerminal)} title="Toggle Terminal">
                 {showTerminal ? 'Hide Term' : 'Show Term'}
               </button>
               <button className="icon-btn" onClick={onSettingsClick} title="Settings">
                 ⚙️
               </button>
             </div>
          </header>
          <div className="chat-container">
            {content}
          </div>
        </main>
      </div>
      <div className={`layout-bottom ${showTerminal ? 'open' : ''}`}>
        {terminal}
      </div>
    </div>
  );
}
