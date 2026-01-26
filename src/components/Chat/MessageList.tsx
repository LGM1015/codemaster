import { useEffect, useRef } from 'react';
import Markdown from 'react-markdown';
import { Message } from '../../types';
import { ToolCallView } from './ToolCall';
import './MessageList.css';

interface MessageListProps {
  messages: Message[];
  streamingContent?: string;
}

export function MessageList({ messages, streamingContent }: MessageListProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  return (
    <div className="message-list">
      {messages.map((msg, idx) => (
        <div key={idx} className={`message ${msg.role}`}>
          <div className="message-avatar">
            {msg.role === 'user' ? '👤' : msg.role === 'assistant' ? '🤖' : '⚙️'}
          </div>
          <div className="message-body">
            {msg.role === 'user' && <div className="message-text">{msg.content}</div>}
            
            {msg.role === 'assistant' && (
              <>
                {msg.content && (
                   <div className="markdown-body">
                     <Markdown>{msg.content}</Markdown>
                   </div>
                )}
                {msg.tool_calls?.map((tc) => (
                  <ToolCallView 
                    key={tc.id} 
                    name={tc.function.name} 
                    args={tc.function.arguments} 
                  />
                ))}
              </>
            )}

            {msg.role === 'tool' && (
              <ToolCallView 
                name={msg.name || 'Unknown Tool'} 
                args="" 
                result={msg.content} 
              />
            )}
          </div>
        </div>
      ))}
      
      {streamingContent && (
        <div className="message assistant streaming">
           <div className="message-avatar">🤖</div>
           <div className="message-body">
             <div className="markdown-body">
               <Markdown>{streamingContent}</Markdown>
             </div>
           </div>
        </div>
      )}
      
      <div ref={bottomRef} />
    </div>
  );
}
