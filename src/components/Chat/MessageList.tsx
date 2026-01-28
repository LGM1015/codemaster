import { useEffect, useRef } from 'react';
import Markdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { Message } from '../../types';
import { ToolCallView } from './ToolCall';
import './MessageList.css';

interface MessageListProps {
  messages: Message[];
  streamingContent?: string;
}

const MarkdownComponents = {
  code(props: any) {
    const { children, className, node, ...rest } = props;
    const match = /language-(\w+)/.exec(className || '');
    return match ? (
      <SyntaxHighlighter
        {...rest}
        PreTag="div"
        children={String(children).replace(/\n$/, '')}
        language={match[1]}
        style={vscDarkPlus}
        customStyle={{ margin: 0, borderRadius: '4px' }}
      />
    ) : (
      <code {...rest} className={className}>
        {children}
      </code>
    );
  }
};

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
                     <Markdown 
                       remarkPlugins={[remarkGfm]}
                       components={MarkdownComponents}
                     >
                       {msg.content}
                     </Markdown>
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
               <Markdown 
                 remarkPlugins={[remarkGfm]}
                 components={MarkdownComponents}
               >
                 {streamingContent}
               </Markdown>
             </div>
           </div>
        </div>
      )}
      
      <div ref={bottomRef} />
    </div>
  );
}
