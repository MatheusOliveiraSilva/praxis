// Configuration
const API_BASE = 'http://localhost:8000';
const USER_ID = 'test_user';

// State
let currentThreadId = null;
let currentToolCalls = new Map(); // Track tool calls by index

// DOM Elements
const threadsList = document.getElementById('threads-list');
const messagesContainer = document.getElementById('messages-container');
const messageForm = document.getElementById('message-form');
const messageInput = document.getElementById('message-input');
const threadTitle = document.getElementById('thread-title');
const newThreadBtn = document.getElementById('new-thread-btn');
const deleteThreadBtn = document.getElementById('delete-thread-btn');

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    loadThreads();
    setupEventListeners();
});

// Event Listeners
function setupEventListeners() {
    newThreadBtn.addEventListener('click', createNewThread);
    deleteThreadBtn.addEventListener('click', deleteCurrentThread);
    messageForm.addEventListener('submit', handleSendMessage);
    
    // Auto-resize textarea
    messageInput.addEventListener('input', (e) => {
        e.target.style.height = 'auto';
        e.target.style.height = e.target.scrollHeight + 'px';
    });
}

// API Calls
async function loadThreads() {
    try {
        const response = await fetch(`${API_BASE}/threads?user_id=${USER_ID}`);
        const data = await response.json();
        
        if (data.threads && data.threads.length > 0) {
            renderThreads(data.threads);
        } else {
            threadsList.innerHTML = '<div class="loading">Nenhuma conversa ainda</div>';
        }
    } catch (error) {
        console.error('Erro ao carregar threads:', error);
        threadsList.innerHTML = '<div class="loading">Erro ao carregar conversas</div>';
    }
}

async function createNewThread() {
    try {
        const response = await fetch(`${API_BASE}/threads`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                user_id: USER_ID,
                title: 'Nova Conversa'
            })
        });
        
        const data = await response.json();
        await loadThreads();
        selectThread(data.thread_id, 'Nova Conversa');
    } catch (error) {
        console.error('Erro ao criar thread:', error);
        alert('Erro ao criar nova conversa');
    }
}

async function deleteCurrentThread() {
    if (!currentThreadId) return;
    
    if (!confirm('Tem certeza que deseja excluir esta conversa?')) {
        return;
    }
    
    try {
        await fetch(`${API_BASE}/threads/${currentThreadId}?user_id=${USER_ID}`, {
            method: 'DELETE'
        });
        
        currentThreadId = null;
        await loadThreads();
        showWelcomeScreen();
    } catch (error) {
        console.error('Erro ao deletar thread:', error);
        alert('Erro ao excluir conversa');
    }
}

async function loadMessages(threadId) {
    try {
        const response = await fetch(`${API_BASE}/threads/${threadId}/messages?user_id=${USER_ID}`);
        const data = await response.json();
        
        renderMessages(data.messages || []);
    } catch (error) {
        console.error('Erro ao carregar mensagens:', error);
        messagesContainer.innerHTML = '<div class="loading">Erro ao carregar mensagens</div>';
    }
}

// Render Functions
function renderThreads(threads) {
    threadsList.innerHTML = threads.map(thread => {
        const date = new Date(thread.created_at).toLocaleDateString('pt-BR', {
            day: '2-digit',
            month: 'short'
        });
        
        return `
            <div class="thread-item ${thread._id === currentThreadId ? 'active' : ''}" 
                 data-id="${thread._id}"
                 onclick="selectThread('${thread._id}', '${escapeHtml(thread.title)}')">
                <div class="thread-title">${escapeHtml(thread.title)}</div>
                <div class="thread-date">${date}</div>
            </div>
        `;
    }).join('');
}

function renderMessages(messages) {
    messagesContainer.innerHTML = '';
    
    messages.forEach(msg => {
        if (msg.role === 'user') {
            appendUserMessage(msg.content);
        } else if (msg.role === 'assistant') {
            appendAssistantMessage(msg.content);
        }
    });
    
    scrollToBottom();
}

function appendUserMessage(content) {
    const messageEl = document.createElement('div');
    messageEl.className = 'message user';
    messageEl.innerHTML = `
        <div class="message-header">
            <span class="message-role">Você</span>
        </div>
        <div class="message-content">${escapeHtml(content)}</div>
    `;
    messagesContainer.appendChild(messageEl);
    scrollToBottom();
}

function appendAssistantMessage(content) {
    const messageEl = document.createElement('div');
    messageEl.className = 'message assistant';
    messageEl.innerHTML = `
        <div class="message-header">
            <span class="message-role">Assistente</span>
        </div>
        <div class="message-content">${escapeHtml(content)}</div>
    `;
    messagesContainer.appendChild(messageEl);
    scrollToBottom();
}

function appendToolCall(index, name, args) {
    // Check if tool call already exists
    let toolEl = currentToolCalls.get(index);
    
    if (!toolEl) {
        toolEl = document.createElement('div');
        toolEl.className = 'tool-call';
        toolEl.innerHTML = `
            <div class="tool-header">
                <svg class="tool-icon running" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="3"></circle>
                    <path d="M12 1v6m0 6v6m5.657-13.657l-4.243 4.243m0 4.828l-4.242 4.243m13.657-5.657l-6 0m-6 0l-6 0m13.657 5.657l-4.243-4.243m0-4.828l-4.242-4.243"></path>
                </svg>
                <span class="tool-name">${escapeHtml(name || 'Carregando...')}</span>
                <span class="tool-status running">Executando</span>
            </div>
            <div class="tool-args"></div>
            <div class="tool-result" style="display: none;"></div>
        `;
        messagesContainer.appendChild(toolEl);
        currentToolCalls.set(index, toolEl);
    }
    
    // Update name if provided
    if (name) {
        const nameEl = toolEl.querySelector('.tool-name');
        nameEl.textContent = name;
    }
    
    // Update arguments
    if (args) {
        const argsEl = toolEl.querySelector('.tool-args');
        const currentArgs = argsEl.textContent || '';
        argsEl.textContent = currentArgs + args;
    }
    
    scrollToBottom();
}

function completeToolCall(index) {
    const toolEl = currentToolCalls.get(index);
    if (toolEl) {
        const statusEl = toolEl.querySelector('.tool-status');
        const iconEl = toolEl.querySelector('.tool-icon');
        
        statusEl.textContent = 'Completo';
        statusEl.className = 'tool-status completed';
        iconEl.classList.remove('running');
        iconEl.innerHTML = `<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline>`;
    }
}

function appendToolResult(result) {
    // Show result in the last tool call
    const lastToolEl = Array.from(currentToolCalls.values()).pop();
    if (lastToolEl) {
        const resultEl = lastToolEl.querySelector('.tool-result');
        
        try {
            const parsed = JSON.parse(result);
            const text = parsed.text || result;
            resultEl.textContent = text.substring(0, 200) + (text.length > 200 ? '...' : '');
        } catch {
            resultEl.textContent = result.substring(0, 200) + (result.length > 200 ? '...' : '');
        }
        
        resultEl.style.display = 'block';
    }
    scrollToBottom();
}

function showTypingIndicator() {
    const indicator = document.createElement('div');
    indicator.id = 'typing-indicator';
    indicator.className = 'message assistant';
    indicator.innerHTML = `
        <div class="message-header">
            <span class="message-role">Assistente</span>
        </div>
        <div class="typing-indicator">
            <span class="typing-dot"></span>
            <span class="typing-dot"></span>
            <span class="typing-dot"></span>
        </div>
    `;
    messagesContainer.appendChild(indicator);
    scrollToBottom();
}

function removeTypingIndicator() {
    const indicator = document.getElementById('typing-indicator');
    if (indicator) {
        indicator.remove();
    }
}

function showWelcomeScreen() {
    messagesContainer.innerHTML = `
        <div class="welcome-screen">
            <div class="welcome-icon">
                <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
                </svg>
            </div>
            <h2>Bem-vindo ao Praxis</h2>
            <p>Crie uma nova conversa ou selecione uma existente para começar</p>
        </div>
    `;
    threadTitle.textContent = 'Selecione uma conversa';
    deleteThreadBtn.style.display = 'none';
    messageInput.disabled = true;
    messageForm.querySelector('.send-btn').disabled = true;
}

// Event Handlers
function selectThread(threadId, title) {
    currentThreadId = threadId;
    threadTitle.textContent = title;
    deleteThreadBtn.style.display = 'flex';
    messageInput.disabled = false;
    messageForm.querySelector('.send-btn').disabled = false;
    
    // Update active state in sidebar
    document.querySelectorAll('.thread-item').forEach(item => {
        item.classList.toggle('active', item.dataset.id === threadId);
    });
    
    loadMessages(threadId);
}

async function handleSendMessage(e) {
    e.preventDefault();
    
    const content = messageInput.value.trim();
    if (!content || !currentThreadId) return;
    
    // Clear input
    messageInput.value = '';
    messageInput.style.height = 'auto';
    
    // Append user message
    appendUserMessage(content);
    
    // Show typing indicator
    showTypingIndicator();
    
    // Reset tool calls tracking
    currentToolCalls.clear();
    
    // Disable input while processing
    messageInput.disabled = true;
    messageForm.querySelector('.send-btn').disabled = true;
    
    try {
        // Send message and handle SSE stream
        const response = await fetch(`${API_BASE}/threads/${currentThreadId}/messages`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'text/event-stream'
            },
            body: JSON.stringify({
                user_id: USER_ID,
                content: content
            })
        });
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        
        removeTypingIndicator();
        
        // Handle SSE stream
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';
        let assistantMessage = '';
        let assistantMessageEl = null;
        
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            
            buffer += decoder.decode(value, { stream: true });
            const lines = buffer.split('\n');
            buffer = lines.pop() || '';
            
            for (const line of lines) {
                if (line.startsWith('event: ')) {
                    const event = line.substring(7).trim();
                    continue;
                }
                
                if (line.startsWith('data: ')) {
                    const data = line.substring(6).trim();
                    if (!data) continue;
                    
                    try {
                        const parsed = JSON.parse(data);
                        
                        // Handle different event types
                        if (parsed.name !== undefined || parsed.arguments !== undefined) {
                            // Tool call event
                            const index = parsed.index || 0;
                            appendToolCall(index, parsed.name, parsed.arguments);
                        } else if (parsed.result !== undefined) {
                            // Tool result event
                            appendToolResult(parsed.result);
                            currentToolCalls.forEach((_, index) => completeToolCall(index));
                        } else if (parsed.content !== undefined) {
                            // Message chunk event
                            if (!assistantMessageEl) {
                                assistantMessageEl = document.createElement('div');
                                assistantMessageEl.className = 'message assistant';
                                assistantMessageEl.innerHTML = `
                                    <div class="message-header">
                                        <span class="message-role">Assistente</span>
                                    </div>
                                    <div class="message-content"></div>
                                `;
                                messagesContainer.appendChild(assistantMessageEl);
                            }
                            
                            assistantMessage += parsed.content;
                            const contentEl = assistantMessageEl.querySelector('.message-content');
                            contentEl.textContent = assistantMessage;
                            scrollToBottom();
                        }
                    } catch (err) {
                        console.error('Error parsing SSE data:', err, data);
                    }
                }
            }
        }
    } catch (error) {
        console.error('Erro ao enviar mensagem:', error);
        removeTypingIndicator();
        alert('Erro ao enviar mensagem. Verifique se a API está rodando.');
    } finally {
        // Re-enable input
        messageInput.disabled = false;
        messageForm.querySelector('.send-btn').disabled = false;
        messageInput.focus();
    }
}

// Utility Functions
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function scrollToBottom() {
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
}

