import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import './SessionList.css';

interface Session {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
}

interface SessionListProps {
  currentSessionId: string | null;
  onSelectSession: (id: string) => void;
  onNewSession: () => void;
  refreshTrigger?: number;
}

export function SessionList({ currentSessionId, onSelectSession, onNewSession, refreshTrigger }: SessionListProps) {
  const { t } = useTranslation();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState('');

  const loadSessions = async () => {
    try {
      const list = await invoke<Session[]>('list_sessions');
      setSessions(list);
    } catch (e) {
      console.error('Failed to load sessions:', e);
    }
  };

  useEffect(() => {
    loadSessions();
  }, [refreshTrigger]);

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!confirm(t('session.deleteConfirm'))) return;
    
    try {
      await invoke('delete_session', { id });
      setSessions(prev => prev.filter(s => s.id !== id));
      if (currentSessionId === id) {
        onNewSession();
      }
    } catch (e) {
      console.error('Failed to delete session:', e);
    }
  };

  const handleRename = async (id: string) => {
    if (!editTitle.trim()) {
      setEditingId(null);
      return;
    }
    
    try {
      await invoke('update_session_title', { id, title: editTitle });
      setSessions(prev => prev.map(s => 
        s.id === id ? { ...s, title: editTitle } : s
      ));
      setEditingId(null);
    } catch (e) {
      console.error('Failed to rename session:', e);
    }
  };

  const startEdit = (session: Session, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingId(session.id);
    setEditTitle(session.title);
  };

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    
    if (days === 0) return t('session.today');
    if (days === 1) return t('session.yesterday');
    if (days < 7) return t('session.daysAgo', { count: days });
    return date.toLocaleDateString();
  };

  return (
    <div className="session-list">
      <div className="session-header">
        <h3>{t('session.title')}</h3>
        <button className="new-session-btn" onClick={onNewSession} title={t('session.new')}>
          +
        </button>
      </div>
      
      <div className="session-items">
        {sessions.length === 0 ? (
          <div className="no-sessions">{t('session.empty')}</div>
        ) : (
          sessions.map(session => (
            <div
              key={session.id}
              className={`session-item ${currentSessionId === session.id ? 'active' : ''}`}
              onClick={() => onSelectSession(session.id)}
            >
              {editingId === session.id ? (
                <input
                  className="session-edit-input"
                  value={editTitle}
                  onChange={(e) => setEditTitle(e.target.value)}
                  onBlur={() => handleRename(session.id)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleRename(session.id);
                    if (e.key === 'Escape') setEditingId(null);
                  }}
                  autoFocus
                  onClick={(e) => e.stopPropagation()}
                />
              ) : (
                <>
                  <div className="session-info">
                    <div className="session-title">{session.title}</div>
                    <div className="session-date">{formatDate(session.updated_at)}</div>
                  </div>
                  <div className="session-actions">
                    <button 
                      className="session-action-btn" 
                      onClick={(e) => startEdit(session, e)}
                      title={t('session.rename')}
                    >
                      ✏️
                    </button>
                    <button 
                      className="session-action-btn delete" 
                      onClick={(e) => handleDelete(session.id, e)}
                      title={t('session.delete')}
                    >
                      🗑️
                    </button>
                  </div>
                </>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
