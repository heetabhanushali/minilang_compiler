/* ============================================
   MiniLang Compiler - Main JavaScript
   Monaco Editor + UI Interactions
   ============================================ */

// Suppress Tailwind CDN warning
console.warn = function() {};

// ============================================
// Global Variables
// ============================================
let editor = null;
let currentOptLevel = 0;
let wasmModule = null;
let wasmCompile = null;
let compileTimeout = null;
let wasmAnalyze = null;
const PISTON_API = 'https://emkc.org/api/v2/piston/execute';
const COMPILE_DEBOUNCE_MS = 300;
const defaultCode = `# Welcome to MiniLang!
# Try compiling this example

func main() {
    let x: int = 10;
    let y: int = 20;
    
    # These will be optimized!
    let sum: int = x + y;
    let product: int = x * 2;  # Strength reduction!
    
    display "Sum: ", sum , "\\n";
    display "Product: ", product;
    
    # Dead code elimination
    if false {
        display "This will be removed!";
    }
}`;

const examplesMetadata = {
    'hello.mini': {
        title: 'Hello World',
        description: 'Basic MiniLang program with display statement'
    },
    'fibonacci.mini': {
        title: 'Fibonacci',
        description: 'Recursive Fibonacci implementation'
    },
    'factorial.mini': {
        title: 'Factorial',
        description: 'Recursive factorial calculation'
    },
    'bubble_sort.mini': {
        title: 'Bubble Sort',
        description: 'Array sorting with bubble sort algorithm'
    },
    'prime_numbers.mini': {
        title: 'Prime Numbers',
        description: 'Find prime numbers with isPrime function'
    }
};

// ============================================
// Initialize Monaco Editor
// ============================================
function initializeEditor() {
    require.config({ 
        paths: { 
            vs: 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs' 
        } 
    });

    require(['vs/editor/editor.main'], function() {
        // Register MiniLang language
        monaco.languages.register({ id: 'minilang' });

        // Define MiniLang syntax highlighting
        monaco.languages.setMonarchTokensProvider('minilang', {
            keywords: [
                'func', 'let', 'const', 'if', 'else', 'while', 'do', 'for',
                'send', 'display', 'int', 'float', 'string', 'bool',
                'true', 'false', 'AND', 'OR', 'NOT', 'break', 'continue'
            ],
            
            operators: [
                '=', '==', '!=', '<', '>', '<=', '>=',
                '+', '-', '*', '/', '%'
            ],

            tokenizer: {
                root: [
                    // Comments
                    [/#.*$/, 'comment'],
                    [/##/, 'comment', '@comment_multiline'],
                    
                    // Keywords
                    [/\b(func|let|const|if|else|while|do|for|send|display|break|continue)\b/, 'keyword'],
                    
                    // Types
                    [/\b(int|float|string|bool)\b/, 'type'],
                    
                    // Logical operators (special highlight)
                    [/\b(AND|OR|NOT)\b/, 'keyword.operator'],
                    
                    // Booleans
                    [/\b(true|false)\b/, 'constant.language'],
                    
                    // Numbers
                    [/\d+\.\d+/, 'number.float'],
                    [/\d+/, 'number'],
                    
                    // Strings
                    [/"([^"\\]|\\.)*$/, 'string.invalid'],
                    [/"/, 'string', '@string'],
                    
                    // Identifiers
                    [/[a-zA-Z_]\w*/, 'identifier'],
                    
                    // Operators
                    [/[+\-*/%<>=!]+/, 'operator'],
                    
                    // Delimiters
                    [/[{}()\[\]]/, '@brackets'],
                    [/[;,:]/, 'delimiter'],
                ]
            ,
                string: [
                    [/[^\\"]+/, 'string'],
                    [/"/, 'string', '@pop']
                ],
                
                comment_multiline: [
                    [/[^#]+/, 'comment'],
                    [/##/, 'comment', '@pop'],
                    [/#/, 'comment']
                ]
            }
        });

        // Define theme
        monaco.editor.defineTheme('minilang-dark', {
            base: 'vs-dark',
            inherit: true,
            rules: [
                { token: 'comment', foreground: '6B7280', fontStyle: 'italic' },
                { token: 'keyword', foreground: 'FBBF24', fontStyle: 'bold' },
                { token: 'keyword.operator', foreground: 'F59E0B', fontStyle: 'bold' },
                { token: 'type', foreground: '06B6D4' },
                { token: 'string', foreground: '10B981' },
                { token: 'number', foreground: 'FB923C' },
                { token: 'number.float', foreground: 'FDBA74' },
                { token: 'constant.language', foreground: 'A78BFA' },
                { token: 'identifier', foreground: 'E2E8F0' },
                { token: 'operator', foreground: 'CBD5E1' }
            ],
            colors: {
                'editor.background': '#0F172A',
                'editor.foreground': '#E2E8F0',
                'editor.lineHighlightBackground': '#1E293B',
                'editorLineNumber.foreground': '#475569',
                'editorLineNumber.activeForeground': '#FBBF24',
                'editor.selectionBackground': '#334155',
                'editorCursor.foreground': '#FBBF24'
            }
        });

        monaco.editor.defineTheme('minilang-light', {
            base: 'vs',
            inherit: true,
            rules: [
                { token: 'comment', foreground: '6B7280', fontStyle: 'italic' },
                { token: 'keyword', foreground: 'D97706', fontStyle: 'bold' },
                { token: 'keyword.operator', foreground: 'F59E0B', fontStyle: 'bold' },
                { token: 'type', foreground: '0891B2' },
                { token: 'string', foreground: '059669' },
                { token: 'number', foreground: 'EA580C' },
                { token: 'number.float', foreground: 'F97316' },
                { token: 'constant.language', foreground: '7C3AED' },
                { token: 'identifier', foreground: '334155' },
                { token: 'operator', foreground: '64748B' }
            ],
            colors: {
                'editor.background': '#FFFFFF',
                'editor.foreground': '#0F172A',
                'editor.lineHighlightBackground': '#F8FAFC',
                'editorLineNumber.foreground': '#94A3B8',
                'editorLineNumber.activeForeground': '#F59E0B',
                'editor.selectionBackground': '#FEF3C7',
                'editorCursor.foreground': '#F59E0B'
            }
        });

        // Create editor
        editor = monaco.editor.create(document.getElementById('editor'), {
            value: defaultCode,
            language: 'minilang',
            theme: 'minilang-dark',
            fontSize: 14,
            fontFamily: "'JetBrains Mono', 'Courier New', monospace",
            lineNumbers: 'on',
            roundedSelection: true,
            scrollBeyondLastLine: false,
            automaticLayout: true,
            minimap: {
                enabled: window.innerWidth > 1024
            },
            scrollbar: {
                verticalScrollbarSize: 10,
                horizontalScrollbarSize: 10
            },
            suggestOnTriggerCharacters: true,
            quickSuggestions: true
        });

        // Add keyboard shortcuts
        editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, function() {
            compileCode();
        });
        editor.addCommand(monaco.KeyMod.Shift | monaco.KeyCode.Enter, function() {
            runCode();
        });
    });
}

// ============================================
// Tab System
// ============================================
function initializeTabs() {
    const navLinks = document.querySelectorAll('.nav-link, .mobile-nav-link');
    const tabContents = document.querySelectorAll('.tab-content');

    navLinks.forEach(link => {
        link.addEventListener('click', () => {
            const targetTab = link.dataset.tab;
            
            // Update active states
            navLinks.forEach(l => l.classList.remove('active'));
            link.classList.add('active');
            
            // Also update the corresponding link in desktop/mobile
            navLinks.forEach(l => {
                if (l.dataset.tab === targetTab) {
                    l.classList.add('active');
                }
            });
            
            // Switch tab content
            tabContents.forEach(content => {
                if (content.id === `${targetTab}-tab`) {
                    content.classList.add('active');
                } else {
                    content.classList.remove('active');
                }
            });
            
            // Close mobile menu if open
            const mobileMenu = document.getElementById('mobile-menu');
            if (mobileMenu) {
                mobileMenu.classList.add('hidden');
            }
            
            // Scroll to top
            window.scrollTo({ top: 0, behavior: 'smooth' });
        });
    });

    // Footer links that trigger tabs
    document.querySelectorAll('.footer-link[data-tab]').forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            const targetTab = link.dataset.tab;
            const targetNavLink = document.querySelector(`.nav-link[data-tab="${targetTab}"]`);
            if (targetNavLink) {
                targetNavLink.click();
            }
        });
    });
}

// ============================================
// Output Tabs
// ============================================
function initializeOutputTabs() {
    const outputTabs = document.querySelectorAll('.output-tab-btn');
    const outputContents = document.querySelectorAll('.output-content');

    outputTabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const targetOutput = tab.dataset.output;
            
            outputTabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            
            outputContents.forEach(content => {
                if (content.id === `output-${targetOutput}`) {
                    content.classList.add('active');
                } else {
                    content.classList.remove('active');
                }
            });
        });
    });
}

// ============================================
// Show/Hide Output Tabs Based on Mode
// ============================================
function setOutputMode(mode) {
    // mode: 'compile' | 'run' | 'analyze' | 'all'
    const allTabs = document.querySelectorAll('.output-tab-btn');
    
    allTabs.forEach(tab => {
        const tabName = tab.dataset.output;
        
        if (mode === 'analyze') {
            // Only show analyze tab
            if (tabName === 'analyze') {
                tab.style.display = '';
                tab.click();
            } else {
                tab.style.display = 'none';
            }
        } else if (mode === 'all') {
            // Show everything
            tab.style.display = '';
        } else {
            // compile / run — show everything except analyze
            if (tabName === 'analyze') {
                tab.style.display = 'none';
            } else {
                tab.style.display = '';
            }
        }
    });
}

// ============================================
// Optimization Level
// ============================================
function initializeOptimizationLevel() {
    const optButtons = document.querySelectorAll('.opt-level-btn');

    optButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            currentOptLevel = parseInt(btn.dataset.level);
            
            optButtons.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            
            console.log(`Optimization level set to: ${currentOptLevel}`);
        });
    });
}

// ============================================
// Compile Button
// ============================================
async function compileCode() {
    const code = editor.getValue();
    
    if (!code.trim()) {
        showError('Please write some code first!');
        return;
    }

    if (!wasmCompile) {
        showError('Compiler not loaded yet. Please wait a moment and try again.');
        return;
    }

    // Show compile/run tabs, hide analyze
    setOutputMode('compile');

    const compileBtn = document.getElementById('compile-btn');
    const originalHTML = compileBtn.innerHTML;
    compileBtn.disabled = true;
    compileBtn.innerHTML = `
        <svg class="animate-spin w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
        </svg>
        <span>Compiling...</span>
    `;

    try {
        const resultJson = wasmCompile(code, currentOptLevel);
        const result = JSON.parse(resultJson);
        
        showCompilationResult(result);
    } catch (error) {
        showError(error.message);
        console.error('Compilation error:', error);
    } finally {
        compileBtn.disabled = false;
        compileBtn.innerHTML = originalHTML;
    }
}

// ============================================
// Run Code (Compile + Execute via Piston API)
// ============================================
async function runCode() {
    const code = editor.getValue();
    
    if (!code.trim()) {
        showError('Please write some code first!');
        return;
    }

    if (!wasmCompile) {
        showError('Compiler not loaded yet. Please wait a moment and try again.');
        return;
    }

    // Show compile/run tabs, hide analyze
    setOutputMode('run');

    const runBtn = document.getElementById('run-btn');
    const originalHTML = runBtn.innerHTML;
    runBtn.disabled = true;
    runBtn.innerHTML = `
        <svg class="animate-spin w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
        </svg>
        <span>Running...</span>
    `;

    // Switch to Result tab
    document.querySelector('.output-tab-btn[data-output="result"]').click();

    try {
        // Step 1: Compile MiniLang → C
        const resultJson = wasmCompile(code, currentOptLevel);
        const result = JSON.parse(resultJson);

        if (!result.success || !result.c_code) {
            showCompilationResult(result);
            return;
        }

        // Update other tabs
        lastCompiledCCode = result.c_code;
        document.getElementById('download-c-btn').classList.remove('hidden');
        document.getElementById('c-code-display').textContent = result.c_code;
        updateStats(result.stats || {});
        updateTokens(result.tokens || []);
        updateAST(result.ast || '');

        // Show "executing" status in Result tab
        showRunStatus('Compiled successfully. Executing on remote server...');

        // Step 2: Execute via Piston API
        const execResult = await executeOnPiston(result.c_code);
        
        // Step 3: Display output in Result tab
        showRunOutput(execResult);

    } catch (error) {
        showRunError('Execution Error', escapeHtml(error.message));
        console.error('Run error:', error);
    } finally {
        runBtn.disabled = false;
        runBtn.innerHTML = originalHTML;
    }
}

// ============================================
// Execute C Code on Piston API
// ============================================
async function executeOnPiston(cCode) {
    const response = await fetch(PISTON_API, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            language: 'c',
            version: '10.2.0',
            files: [{
                name: 'main.c',
                content: cCode
            }],
            stdin: '',
            args: [],
            compile_timeout: 10000,
            run_timeout: 5000,
            compile_memory_limit: -1,
            run_memory_limit: -1,
        })
    });

    if (!response.ok) {
        throw new Error(`Piston API error: ${response.status} ${response.statusText}`);
    }

    return await response.json();
}

// ============================================
// Display Run Output in Result Tab
// ============================================
function showRunOutput(pistonResult) {
    const resultDiv = document.getElementById('output-result');
    
    const compile = pistonResult.compile || {};
    const run = pistonResult.run || {};
    
    // GCC compilation failed
    if (compile.code !== 0 && compile.code !== undefined) {
        resultDiv.innerHTML = `
            <div class="p-6">
                <div class="message-error">
                    <svg class="w-6 h-6 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                    </svg>
                    <span class="font-semibold">GCC Compilation Failed</span>
                </div>
                <div class="mt-4 p-4 rounded-lg" style="background: var(--bg-secondary); border: 1px solid var(--border);">
                    <pre class="text-sm font-mono" style="color: var(--error); white-space: pre-wrap; background: transparent; border: none; padding: 0; margin: 0;">${escapeHtml(compile.stderr || compile.output || 'Unknown GCC error')}</pre>
                </div>
            </div>
        `;
        return;
    }

    // Build output text
    let outputParts = [];
    
    if (run.stdout) {
        outputParts.push(run.stdout);
    }
    
    if (run.stderr) {
        outputParts.push(run.stderr);
    }
    
    if (run.signal) {
        const signalMessages = {
            'SIGSEGV': '⚠ Segmentation fault (invalid memory access)',
            'SIGFPE': '⚠ Floating point exception (division by zero?)',
            'SIGABRT': '⚠ Program aborted',
            'SIGKILL': '⚠ Program killed (timeout or memory limit)',
            'SIGTERM': '⚠ Program terminated',
        };
        outputParts.push(signalMessages[run.signal] || `⚠ Signal: ${run.signal}`);
    }

    const exitCode = run.code ?? 0;
    const isSuccess = exitCode === 0 && !run.signal;
    const outputText = outputParts.join('\n') || '(no output)';

    resultDiv.innerHTML = `
        <div class="p-6">
            <div class="${isSuccess ? 'message-success' : 'message-error'}">
                <svg class="w-6 h-6 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    ${isSuccess 
                        ? '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>'
                        : '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>'
                    }
                </svg>
                <span class="font-semibold">${isSuccess ? 'Program executed successfully' : 'Program exited with errors'}</span>
            </div>
            <div class="mt-4 p-4 rounded-lg" style="background: var(--bg-secondary); border: 1px solid var(--border);">
                <pre class="text-sm font-mono" style="color: var(--text-primary); white-space: pre-wrap; background: transparent; border: none; padding: 0; margin: 0;">${escapeHtml(outputText)}</pre>
                <div class="mt-3 pt-3 text-xs" style="border-top: 1px solid var(--border); color: ${isSuccess ? 'var(--success)' : 'var(--error)'};">
                    ${isSuccess ? '✓' : '✗'} Process exited with code ${exitCode}${run.signal ? ` (${run.signal})` : ''}
                </div>
            </div>
        </div>
    `;
}

// ============================================
// Run Helper Functions
// ============================================
function showRunStatus(message) {
    const resultDiv = document.getElementById('output-result');
    resultDiv.innerHTML = `
        <div class="p-6 h-full flex items-center justify-center">
            <div class="text-center">
                <div class="loader mx-auto mb-4" style="border-top-color: #10B981;"></div>
                <p class="text-sm dark:text-slate-400 text-slate-600">${escapeHtml(message)}</p>
            </div>
        </div>
    `;
}

function showRunError(title, htmlContent) {
    const resultDiv = document.getElementById('output-result');
    resultDiv.innerHTML = `
        <div class="p-6">
            <div class="message-error">
                <svg class="w-6 h-6 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                </svg>
                <span class="font-semibold">${escapeHtml(title)}</span>
            </div>
            <div class="mt-4 p-4 rounded-lg" style="background: var(--bg-secondary); border: 1px solid var(--border);">
                <pre class="text-sm font-mono" style="color: var(--error); white-space: pre-wrap; background: transparent; border: none; padding: 0; margin: 0;">${htmlContent}</pre>
            </div>
        </div>
    `;
}

// ============================================
// ANSI to HTML Converter
// ============================================
function ansiToHtml(ansiString) {
    if (!ansiString) return '';
    
    const COLORS = {
        '30': 'ansi-black', '31': 'ansi-red', '32': 'ansi-green', '33': 'ansi-yellow',
        '34': 'ansi-blue', '35': 'ansi-magenta', '36': 'ansi-cyan', '37': 'ansi-white',
        '90': 'ansi-bright-black', '91': 'ansi-bright-red', '92': 'ansi-bright-green',
        '93': 'ansi-bright-yellow', '94': 'ansi-bright-blue', '95': 'ansi-bright-magenta',
        '96': 'ansi-bright-cyan', '97': 'ansi-bright-white',
        '1': 'ansi-bold', '2': 'ansi-dim', '3': 'ansi-italic', '4': 'ansi-underline'
    };
    
    let result = '';
    let openTags = [];
    let i = 0;

    while (i < ansiString.length) {
        if ((ansiString.charCodeAt(i) === 0x1b) && ansiString[i + 1] === '[') {
            let j = i + 2;
            while (j < ansiString.length && ansiString[j] !== 'm') j++;
            
            if (j < ansiString.length) {
                const codes = ansiString.substring(i + 2, j).split(';');
                for (const code of codes) {
                    if (code === '0' || code === '') {
                        while (openTags.length) { result += '</span>'; openTags.pop(); }
                    } else if (COLORS[code]) {
                        result += `<span class="${COLORS[code]}">`;
                        openTags.push(code);
                    }
                }
                i = j + 1;
                continue;
            }
        }
        
        const c = ansiString[i];
        if (c === '<') result += '&lt;';
        else if (c === '>') result += '&gt;';
        else if (c === '&') result += '&amp;';
        else result += c;
        i++;
    }
    
    while (openTags.length) { result += '</span>'; openTags.pop(); }
    return result;
}

function showCompilationResult(result) {
    const resultDiv = document.getElementById('output-result');
    
    if (result.success && result.c_code) {
        lastCompiledCCode = result.c_code;
        document.getElementById('download-c-btn').classList.remove('hidden');
    } else {
        document.getElementById('download-c-btn').classList.add('hidden');
    }
    
    if (result.success) {
        resultDiv.innerHTML = `
            <div class="p-6">
                <div class="message-success">
                    <svg class="w-6 h-6 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                    </svg>
                    <span class="font-semibold">Compilation Successful</span>
                </div>
                <div class="mt-4 p-4 rounded-lg" style="background: var(--bg-secondary); border: 1px solid var(--border);">
                    <pre class="text-sm font-mono" style="color: var(--text-primary); background: transparent;">Optimization level: ${currentOptLevel}\n\nClick Run to execute your program.</pre>
                </div>
            </div>
        `;
    } else {
        const errorContent = result.error_ansi 
            ? ansiToHtml(result.error_ansi) 
            : escapeHtml(result.error || 'Unknown error');

        resultDiv.innerHTML = `
            <div class="p-6">
                <div class="message-error mb-4">
                    <svg class="w-6 h-6 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                    </svg>
                    <span class="font-semibold">Compilation Failed</span>
                </div>
                <div class="error-console">
                    <pre>${errorContent}</pre>
                </div>
            </div>
        `;
    }

    document.getElementById('c-code-display').textContent = result.c_code || '// No C code generated';
    updateStats(result.stats || {});
    updateTokens(result.tokens || []);
    updateAST(result.ast || '// No AST generated');
    document.querySelector('.output-tab-btn[data-output="result"]').click();
}

// ============================================
// Parse Error Details for Better Display
// ============================================
function parseErrorDetails(errorMessage) {
    const offsetMatch = errorMessage.match(/offset:\s*SourceOffset\((\d+)\)/i) || 
                        errorMessage.match(/SourceOffset\((\d+)\)/);
    const offset = offsetMatch ? parseInt(offsetMatch[1]) : null;
    
    // Calculate line and column from offset
    let location = null;
    if (offset !== null && editor) {
        const code = editor.getValue();
        const lines = code.split('\n');
        let currentOffset = 0;
        
        for (let i = 0; i < lines.length; i++) {
            const lineLength = lines[i].length + 1;
            if (currentOffset + lineLength > offset) {
                const columnInLine = offset - currentOffset;
                location = {
                    line: i + 1,
                    column: columnInLine
                };
                break;
            }
            currentOffset += lineLength;
        }
    }
    
    // Clean up the error message
    let cleanMessage = errorMessage;
    
    // Replace token names with actual symbols
    const tokenReplacements = {
        'Semicolon': ';',
        'Comma': ',',
        'Colon': ':',
        'LeftParen': '(',
        'RightParen': ')',
        'LeftBrace': '{',
        'RightBrace': '}',
        'LeftBracket': '[',
        'RightBracket': ']',
        'Equals': '=',
        'Equal': '=',
        'Assign': '=',
        'Plus': '+',
        'Minus': '-',
        'Star': '*',
        'Slash': '/',
        'Percent': '%',
        'Arrow': '->',
        'DoubleEquals': '==',
        'NotEquals': '!=',
        'LessThan': '<',
        'GreaterThan': '>',
        'LessEquals': '<=',
        'GreaterEquals': '>=',
        'Let': 'let',       
        'Func': 'func',  
        'If': 'if',       
        'Else': 'else',    
        'While': 'while',    
        'For': 'for',       
        'Display': 'display', 
        'Send': 'send', 
    };
    
    // Replace "expected: "TokenName"" with "expected: 'symbol'"
    cleanMessage = cleanMessage.replace(/expected:\s*"(\w+)"/g, (match, token) => {
        const symbol = tokenReplacements[token] || token.toLowerCase();
        return `expected: '${symbol}'`;
    });
    
    // Replace "found: "TokenName"" with "found: 'symbol'"
    cleanMessage = cleanMessage.replace(/found:\s*"(\w+)"/g, (match, token) => {
        const symbol = tokenReplacements[token] || token.toLowerCase();
        return `found: '${symbol}'`;
    });
    
    // Clean up the span information (remove it for cleaner display)
    cleanMessage = cleanMessage.replace(/,?\s*span:\s*SourceSpan\s*{[^}]+}/gi, '');
    
    // Clean up other verbose parts
    cleanMessage = cleanMessage.replace(/UnexpectedToken\s*{/, 'Unexpected Token: {');
    cleanMessage = cleanMessage.replace(/Parser Error:\s*/i, 'Parser Error: ');
    cleanMessage = cleanMessage.replace(/Lexer Error:\s*/i, 'Lexer Error: ');
    cleanMessage = cleanMessage.replace(/Type Error:\s*/i, 'Type Error: ');
    
    return {
        location: location,
        cleanMessage: cleanMessage,
        originalMessage: errorMessage
    };
}

// ============================================
// Update Statistics
// ============================================
function updateStats(stats) {
    const total = (stats.constants_folded || 0) + 
                  (stats.dead_code_removed || 0) + 
                  (stats.constants_propagated || 0) + 
                  (stats.strength_reductions || 0);

    const maxValue = Math.max(total, 10); // For bar scaling

    // Update values
    document.getElementById('stat-constants').textContent = stats.constants_folded || 0;
    document.getElementById('stat-deadcode').textContent = stats.dead_code_removed || 0;
    document.getElementById('stat-propagated').textContent = stats.constants_propagated || 0;
    document.getElementById('stat-strength').textContent = stats.strength_reductions || 0;
    document.getElementById('stat-total').textContent = total;

    // Update bars
    const updateBar = (id, value) => {
        const bar = document.getElementById(id);
        const percentage = maxValue > 0 ? (value / maxValue * 100) : 0;
        if (bar) bar.style.width = percentage + '%';
    };

    updateBar('bar-constants', stats.constants_folded || 0);
    updateBar('bar-deadcode', stats.dead_code_removed || 0);
    updateBar('bar-propagated', stats.constants_propagated || 0);
    updateBar('bar-strength', stats.strength_reductions || 0);
}

// ============================================
// Update Tokens Display
// ============================================
function updateTokens(tokens) {
    const tbody = document.getElementById('tokens-table');
    const countSpan = document.getElementById('token-count');
    
    if (!tokens || tokens.length === 0) {
        tbody.innerHTML = `
            <tr>
                <td colspan="5" class="text-center p-8 dark:text-slate-500 text-slate-400">
                    No tokens available
                </td>
            </tr>
        `;
        countSpan.textContent = '0 tokens';
        return;
    }
    
    countSpan.textContent = `${tokens.length} tokens`;
    
    tbody.innerHTML = tokens.map((token, idx) => `
        <tr class="border-b dark:border-slate-800 border-slate-200 dark:hover:bg-slate-700 hover:bg-amber-50 transition">
            <td class="p-2 dark:text-slate-500 text-slate-400">${idx + 1}</td>
            <td class="p-2 text-primary font-semibold">${escapeHtml(token.token_type)}</td>
            <td class="p-2 dark:text-slate-300 text-slate-900 font-medium">${escapeHtml(token.value)}</td>
            <td class="p-2 dark:text-slate-400 text-slate-800">${token.line}</td>
            <td class="p-2 dark:text-slate-400 text-slate-800">${token.column}</td>
        </tr>
    `).join('');
}

// ============================================
// Update AST Display
// ============================================
function updateAST(ast) {
    const astDisplay = document.getElementById('ast-display');
    
    // Try to parse as JSON
    try {
        const astObject = JSON.parse(ast);
        astDisplay.innerHTML = '';
        const tree = createTreeView(astObject, 'Program', 0);
        astDisplay.appendChild(tree);
        
        // Add expand/collapse all functionality
        setupExpandCollapseButtons();
    } catch (error) {
        // If not JSON, display as plain text
        astDisplay.innerHTML = `<div class="ast-placeholder">${escapeHtml(ast)}</div>`;
    }
}

function createTreeView(data, key = 'root', depth = 0) {
    const container = document.createElement('div');
    container.className = 'ast-tree-container';
    container.style.marginLeft = depth > 0 ? '1.25rem' : '0';
    
    const isObject = typeof data === 'object' && data !== null;
    const isArray = Array.isArray(data);
    const isEmpty = isObject && Object.keys(data).length === 0;
    
    if (isEmpty) {
        const emptyNode = document.createElement('div');
        emptyNode.className = 'ast-node-line';
        emptyNode.innerHTML = `
            <span class="ast-key">${escapeHtml(key)}</span>
            <span class="ast-colon">:</span>
            <span class="ast-value-empty">{}</span>
        `;
        container.appendChild(emptyNode);
        return container;
    }
    
    if (!isObject) {
        // Leaf node (primitive value)
        const leafNode = document.createElement('div');
        leafNode.className = 'ast-node-line';
        
        const valueClass = typeof data === 'string' ? 'ast-value-string' :
                          typeof data === 'number' ? 'ast-value-number' :
                          typeof data === 'boolean' ? 'ast-value-boolean' :
                          'ast-value-null';
        
        const displayValue = typeof data === 'string' ? `"${escapeHtml(data)}"` : String(data);
        
        leafNode.innerHTML = `
            <span class="ast-key">${escapeHtml(key)}</span>
            <span class="ast-colon">:</span>
            <span class="${valueClass}">${displayValue}</span>
        `;
        container.appendChild(leafNode);
        return container;
    }
    
    // Branch node (object or array)
    const branchNode = document.createElement('div');
    branchNode.className = 'ast-node-branch';
    
    const header = document.createElement('div');
    header.className = 'ast-node-header';
    
    const toggle = document.createElement('span');
    toggle.className = 'ast-toggle';
    toggle.textContent = '▼';
    toggle.setAttribute('data-expanded', 'true');
    
    const keySpan = document.createElement('span');
    keySpan.className = 'ast-key-branch';
    keySpan.textContent = key;
    
    const bracket = document.createElement('span');
    bracket.className = 'ast-bracket';
    bracket.textContent = isArray ? ' [' : ' {';
    
    header.appendChild(toggle);
    header.appendChild(keySpan);
    header.appendChild(bracket);
    
    const content = document.createElement('div');
    content.className = 'ast-node-content';
    
    // Add children
    if (isArray) {
        data.forEach((item, index) => {
            content.appendChild(createTreeView(item, `[${index}]`, depth + 1));
        });
    } else {
        Object.entries(data).forEach(([childKey, childValue]) => {
            content.appendChild(createTreeView(childValue, childKey, depth + 1));
        });
    }
    
    const closeBracket = document.createElement('div');
    closeBracket.className = 'ast-close-bracket';
    closeBracket.textContent = isArray ? ']' : '}';
    closeBracket.style.marginLeft = depth > 0 ? '1.25rem' : '0';
    
    // Toggle functionality
    toggle.addEventListener('click', (e) => {
        e.stopPropagation();
        const isExpanded = toggle.getAttribute('data-expanded') === 'true';
        
        if (isExpanded) {
            content.style.display = 'none';
            closeBracket.style.display = 'none';
            toggle.textContent = '▶';
            toggle.setAttribute('data-expanded', 'false');
            bracket.textContent = isArray ? ' [...' : ' {...';
            
            const summary = document.createElement('span');
            summary.className = 'ast-collapsed-summary';
            summary.textContent = isArray ? `${data.length} items]` : `${Object.keys(data).length} fields}`;
            header.appendChild(summary);
        } else {
            content.style.display = 'block';
            closeBracket.style.display = 'block';
            toggle.textContent = '▼';
            toggle.setAttribute('data-expanded', 'true');
            bracket.textContent = isArray ? ' [' : ' {';
            
            const summary = header.querySelector('.ast-collapsed-summary');
            if (summary) summary.remove();
        }
    });
    
    header.style.cursor = 'pointer';
    header.addEventListener('click', () => toggle.click());
    
    branchNode.appendChild(header);
    branchNode.appendChild(content);
    container.appendChild(branchNode);
    container.appendChild(closeBracket);
    
    return container;
}

function setupExpandCollapseButtons() {
    const expandBtn = document.getElementById('expand-all-btn');
    const collapseBtn = document.getElementById('collapse-all-btn');
    
    if (expandBtn) {
        expandBtn.onclick = () => {
            document.querySelectorAll('.ast-toggle').forEach(toggle => {
                if (toggle.getAttribute('data-expanded') === 'false') {
                    toggle.click();
                }
            });
        };
    }
    
    if (collapseBtn) {
        collapseBtn.onclick = () => {
            document.querySelectorAll('.ast-toggle').forEach(toggle => {
                if (toggle.getAttribute('data-expanded') === 'true') {
                    toggle.click();
                }
            });
        };
    }
}

// ============================================
// HTML Escape Utility
// ============================================
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ============================================
// Show Error
// ============================================
function showError(message) {
    const resultDiv = document.getElementById('output-result');
    resultDiv.innerHTML = `
        <div class="p-6">
            <div class="message-error">
                <svg class="w-6 h-6 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                <div>
                    <p class="font-semibold mb-2">Error</p>
                    <p class="text-sm">${message}</p>
                </div>
            </div>
        </div>
    `;
}

// ============================================
// Analyze Code
// ============================================
async function analyzeCode() {
    const code = editor.getValue();
    
    if (!code.trim()) {
        showAnalyzeError('Please write some code first!');
        return;
    }

    if (!wasmAnalyze) {
        showAnalyzeError('Analyzer not loaded yet. Please wait a moment and try again.');
        return;
    }

    // Show only analyze tab
    setOutputMode('analyze');

    const analyzeBtn = document.getElementById('analyze-btn');
    const originalHTML = analyzeBtn.innerHTML;
    analyzeBtn.disabled = true;
    analyzeBtn.innerHTML = `
        <svg class="animate-spin w-4 h-4 lg:w-5 lg:h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
        </svg>
        <span>Analyzing...</span>
    `;

    try {
        const resultJson = wasmAnalyze(code);
        const result = JSON.parse(resultJson);
        
        displayAnalysisResult(result);
    } catch (error) {
        showAnalyzeError(error.message);
        console.error('Analysis error:', error);
    } finally {
        analyzeBtn.disabled = false;
        analyzeBtn.innerHTML = originalHTML;
    }
}

// ============================================
// Display Analysis Result
// ============================================
function displayAnalysisResult(result) {
    const display = document.getElementById('analyze-display');
    
    if (!result.success) {
        const errorContent = result.error_ansi 
            ? ansiToHtml(result.error_ansi) 
            : escapeHtml(result.error || 'Unknown error');
        
        display.innerHTML = `
            <div class="message-error mb-4">
                <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                </svg>
                <span class="font-semibold">Analysis Failed</span>
            </div>
            <div class="error-console">
                <pre>${errorContent}</pre>
            </div>
        `;
        return;
    }

    const report = result.report;
    if (!report) {
        display.innerHTML = `<div class="text-center py-8 dark:text-slate-400 text-slate-600">No analysis data available</div>`;
        return;
    }

    const totals = report.program_totals;
    
    // Build HTML
    let html = `
        <!-- Overall Rating Banner -->
        <div class="analyze-banner analyze-banner-${totals.overall_rating.toLowerCase()}">
            <div class="analyze-banner-rating">
                <span class="analyze-banner-grade">${totals.overall_rating}</span>
                <span class="analyze-banner-label">${getRatingLabel(totals.overall_rating)}</span>
            </div>
            <div class="analyze-banner-stats">
                <div class="analyze-banner-stat">
                    <span class="value">${totals.total_functions}</span>
                    <span class="label">Functions</span>
                </div>
                <div class="analyze-banner-stat">
                    <span class="value">${totals.total_loc}</span>
                    <span class="label">LOC</span>
                </div>
                <div class="analyze-banner-stat">
                    <span class="value">${totals.avg_cyclomatic.toFixed(1)}</span>
                    <span class="label">Avg Cyclo</span>
                </div>
                <div class="analyze-banner-stat">
                    <span class="value">${totals.avg_cognitive.toFixed(1)}</span>
                    <span class="label">Avg Cog</span>
                </div>
            </div>
        </div>

        <!-- Functions -->
        <div class="analyze-functions-header">
            <span>Function Details</span>
            <span class="analyze-functions-count">${report.functions.length} function${report.functions.length !== 1 ? 's' : ''}</span>
        </div>
    `;

    // Function Cards
    for (const func of report.functions) {
        const warnings = getFunctionWarnings(func);
        const cycloPercent = Math.min((func.cyclomatic_complexity / 20) * 100, 100);
        const cogPercent = Math.min((func.cognitive_complexity / 25) * 100, 100);
        
        html += `
            <div class="analyze-func-card">
                <div class="analyze-func-header">
                    <div class="analyze-func-name">
                        <span class="analyze-func-badge analyze-badge-${func.rating.toLowerCase()}">${func.rating}</span>
                        <code>${escapeHtml(func.name)}</code>
                        <span class="analyze-func-params">(${func.parameter_count} param${func.parameter_count !== 1 ? 's' : ''})</span>
                    </div>
                    <div class="analyze-func-loc">${func.loc} lines</div>
                </div>
                
                <div class="analyze-func-metrics">
                    <div class="analyze-func-metric">
                        <div class="analyze-func-metric-header">
                            <span>Cyclomatic</span>
                            <span class="${getComplexityClass(func.cyclomatic_complexity, 'cyclomatic')}">${func.cyclomatic_complexity}</span>
                        </div>
                        <div class="analyze-progress">
                            <div class="analyze-progress-fill ${getBarColorClass(func.cyclomatic_complexity, 'cyclomatic')}" style="width: ${cycloPercent}%"></div>
                        </div>
                    </div>
                    <div class="analyze-func-metric">
                        <div class="analyze-func-metric-header">
                            <span>Cognitive</span>
                            <span class="${getComplexityClass(func.cognitive_complexity, 'cognitive')}">${func.cognitive_complexity}</span>
                        </div>
                        <div class="analyze-progress">
                            <div class="analyze-progress-fill ${getBarColorClass(func.cognitive_complexity, 'cognitive')}" style="width: ${cogPercent}%"></div>
                        </div>
                    </div>
                </div>

                <div class="analyze-func-details">
                    <div class="analyze-detail">
                        <span class="analyze-detail-value">${func.statement_count}</span>
                        <span class="analyze-detail-label">Statements</span>
                    </div>
                    <div class="analyze-detail">
                        <span class="analyze-detail-value">${func.max_nesting_depth}</span>
                        <span class="analyze-detail-label">Max Depth</span>
                    </div>
                    <div class="analyze-detail">
                        <span class="analyze-detail-value">${func.fan_out}</span>
                        <span class="analyze-detail-label">Fan-out</span>
                    </div>
                    <div class="analyze-detail">
                        <span class="analyze-detail-value">${func.halstead.volume.toFixed(0)}</span>
                        <span class="analyze-detail-label">Volume</span>
                    </div>
                </div>
                
                ${warnings.length > 0 ? `
                    <div class="analyze-func-warnings">
                        ${warnings.map(w => `
                            <div class="analyze-func-warning">
                                <span class="warning-icon">⚠</span>
                                <span>${escapeHtml(w)}</span>
                            </div>
                        `).join('')}
                    </div>
                ` : ''}
            </div>
        `;
    }

    display.innerHTML = html;
}

function getRatingLabel(rating) {
    const labels = {
        'A': 'Excellent',
        'B': 'Good',
        'C': 'Moderate',
        'D': 'Complex',
        'F': 'Very Complex'
    };
    return labels[rating] || 'Unknown';
}

function getComplexityClass(value, type) {
    if (type === 'cyclomatic') {
        if (value <= 5) return 'complexity-good';
        if (value <= 10) return 'complexity-ok';
        if (value <= 20) return 'complexity-warning';
        return 'complexity-danger';
    } else { // cognitive
        if (value <= 5) return 'complexity-good';
        if (value <= 10) return 'complexity-ok';
        if (value <= 15) return 'complexity-warning';
        return 'complexity-danger';
    }
}

function getBarColorClass(value, type) {
    if (type === 'cyclomatic') {
        if (value <= 5) return 'bar-good';
        if (value <= 10) return 'bar-ok';
        if (value <= 20) return 'bar-warning';
        return 'bar-danger';
    } else {
        if (value <= 5) return 'bar-good';
        if (value <= 10) return 'bar-ok';
        if (value <= 15) return 'bar-warning';
        return 'bar-danger';
    }
}

function getFunctionWarnings(func) {
    const warnings = [];
    
    if (func.cyclomatic_complexity > 10) {
        warnings.push(`High cyclomatic complexity (${func.cyclomatic_complexity}). Consider breaking into smaller functions.`);
    }
    
    if (func.cognitive_complexity > 15) {
        warnings.push(`High cognitive complexity (${func.cognitive_complexity}). This function may be hard to understand.`);
    }
    
    if (func.max_nesting_depth > 3) {
        warnings.push(`Deep nesting (depth ${func.max_nesting_depth}). Consider using early returns.`);
    }
    
    if (func.parameter_count > 5) {
        warnings.push(`Too many parameters (${func.parameter_count}). Consider grouping related parameters.`);
    }
    
    if (func.fan_out > 8) {
        warnings.push(`High fan-out (${func.fan_out}). This function depends on many others.`);
    }
    
    if (func.loc > 50) {
        warnings.push(`Long function (${func.loc} LOC). Consider splitting into smaller functions.`);
    }
    
    return warnings;
}

function showAnalyzeError(message) {
    const display = document.getElementById('analyze-display');
    display.innerHTML = `
        <div class="message-error">
            <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
            <span>${escapeHtml(message)}</span>
        </div>
    `;
}

// ============================================
// Loading Overlay
// ============================================
function showLoading(show) {
    // Disabled for now - causes UI to freeze
    // return;
}

// ============================================
// Clear Button
// ============================================
function clearEditor() {
    if (confirm('Are you sure you want to clear the editor?')) {
        editor.setValue(defaultCode);
        
        // Hide download C button
        document.getElementById('download-c-btn').classList.add('hidden');
        lastCompiledCCode = null;
        
        // Reset output
        const resultDiv = document.getElementById('output-result');
        resultDiv.innerHTML = `<div class="p-4 lg:p-6 h-full flex items-center justify-center"><div class="text-center dark:text-slate-400 text-slate-600"><div class="relative mx-auto mb-4 w-16 h-16 lg:w-20 lg:h-20"><div class="absolute inset-0 bg-primary/10 rounded-2xl animate-pulse"></div><svg class="w-16 h-16 lg:w-20 lg:h-20 relative z-10 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/></svg></div>
        <p class="text-lg font-medium dark:text-slate-300 text-slate-700">Playground is ready</p>
        <p class="text-sm mt-4 dark:text-slate-500 text-slate-500">
        press <kbd class="px-2 py-1 dark:bg-slate-800 bg-slate-100 rounded border dark:border-slate-700 border-slate-300 dark:text-slate-300 text-slate-700">Ctrl</kbd> + <kbd class="px-2 py-1 dark:bg-slate-800 bg-slate-100 rounded border dark:border-slate-700 border-slate-300 dark:text-slate-300 text-slate-700">Enter</kbd> to compile</p>
        <br>
        <p class="text-sm mt-2 dark:text-slate-500 text-slate-500">
        press <kbd class="px-2 py-1 dark:bg-slate-800 bg-slate-100 rounded border dark:border-slate-700 border-slate-300 dark:text-slate-300 text-slate-700">Shift</kbd> + <kbd class="px-2 py-1 dark:bg-slate-800 bg-slate-100 rounded border dark:border-slate-700 border-slate-300 dark:text-slate-300 text-slate-700">Enter</kbd> to compile and execute</p></div></div>`;
        
        // Clear C code tab
        document.getElementById('c-code-display').textContent = '// Generated C code will appear here after compilation';
        
        // Clear tokens tab
        document.getElementById('tokens-table').innerHTML = `
            <tr>
                <td colspan="5" class="text-center p-8 dark:text-slate-500 text-slate-400">
                    No tokens yet. Compile your code first.
                </td>
            </tr>
        `;
        document.getElementById('token-count').textContent = '0 tokens';
        
        // Clear AST tab
        document.getElementById('ast-display').innerHTML = '<div class="ast-placeholder">// AST will appear here after compilation</div>';
        
        // Clear Analyze tab
        document.getElementById('analyze-display').innerHTML = `
            <div class="text-center py-12 dark:text-slate-500 text-slate-400">
                <svg class="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
                </svg>
                <p class="text-sm">Click "Analyze" button to run static analysis</p>
                <p class="text-xs mt-2 dark:text-slate-600 text-slate-400">Calculates complexity metrics for your code</p>
            </div>
        `;
        
        // Reset stats
        updateStats({
            constants_folded: 0,
            dead_code_removed: 0,
            constants_propagated: 0,
            strength_reductions: 0
        });

        // Show all tabs except analyze (default state)
        setOutputMode('all');
        
        // Switch to result tab
        document.querySelector('.output-tab-btn[data-output="result"]').click();
    }
}

// ============================================
// Download Button
// ============================================
let lastCompiledCCode = null;

function downloadSource() {
    const code = editor.getValue();
    const blob = new Blob([code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'program.mini';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

function downloadCCode() {
    if (!lastCompiledCCode) {
        showError('No C code available. Please compile first.');
        return;
    }
    
    const blob = new Blob([lastCompiledCCode], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'program.c';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

// ============================================
// Theme Toggle
// ============================================
function initializeThemeToggle() {
    const themeToggle = document.getElementById('theme-toggle');
    const html = document.documentElement;
    
    themeToggle.addEventListener('click', () => {
        if (html.classList.contains('dark')) {
            // Switch to light mode
            html.classList.remove('dark');
            html.classList.add('light');
            
            themeToggle.querySelector('.dark-icon').classList.add('hidden');
            themeToggle.querySelector('.light-icon').classList.remove('hidden');
            
            // Change Monaco editor theme
            if (editor) {
                monaco.editor.setTheme('minilang-light');
            }
        } else {
            // Switch to dark mode
            html.classList.remove('light');
            html.classList.add('dark');
            
            themeToggle.querySelector('.light-icon').classList.add('hidden');
            themeToggle.querySelector('.dark-icon').classList.remove('hidden');
            
            // Change Monaco editor theme
            if (editor) {
                monaco.editor.setTheme('minilang-dark');
            }
        }
    });
}

// ============================================
// Mobile Menu
// ============================================
function initializeMobileMenu() {
    const mobileMenuBtn = document.getElementById('mobile-menu-btn');
    const mobileMenu = document.getElementById('mobile-menu');
    
    if (mobileMenuBtn && mobileMenu) {
        mobileMenuBtn.addEventListener('click', () => {
            mobileMenu.classList.toggle('hidden');
        });
    }
}

// ============================================
// Load Examples
// ============================================
async function loadExamples() {
    const examplesGrid = document.getElementById('examples-grid');
    
    for (const [filename, metadata] of Object.entries(examplesMetadata)) {
        const card = document.createElement('div');
        card.className = 'example-card';
        card.innerHTML = `
            <h3 class="example-card-title">${metadata.title}</h3>
            <p class="example-card-desc">${metadata.description}</p>
            <div class="example-card-code">
                <code>Loading...</code>
            </div>
        `;
        
        examplesGrid.appendChild(card);
        
        // Load the actual file
        try {
            const response = await fetch(`examples/${filename}`);
            const code = await response.text();
            
            // Update the preview
            const preview = code.split('\n').slice(0, 5).join('\n');
            card.querySelector('.example-card-code code').textContent = 
                preview + (code.split('\n').length > 5 ? '\n...' : '');
            
            // Add click handler
            card.addEventListener('click', () => {
                if (editor) {
                    editor.setValue(code);
                    document.querySelector('.nav-link[data-tab="playground"]').click();
                    window.scrollTo({ top: 0, behavior: 'smooth' });
                }
            });
        } catch (error) {
            console.error(`Failed to load ${filename}:`, error);
            card.querySelector('.example-card-code code').textContent = 
                'Failed to load example';
        }
    }
}

// ============================================
// Event Listeners
// ============================================
document.addEventListener('DOMContentLoaded', async () => {

    try {
        const wasm = await import('./pkg/minilang_compiler.js');
        await wasm.default(); // Initialize WASM
        wasm.init_panic_hook(); // Better error messages
        
        wasmModule = wasm;
        wasmCompile = wasm.compile;
        wasmAnalyze = wasm.analyze;
    } catch (error) {
        console.error('Failed to load WASM:', error);
        alert('Failed to load compiler. Please refresh the page.');
        return;
    }
    // Initialize Monaco Editor
    initializeEditor();
    
    // Initialize UI components
    initializeTabs();
    initializeOutputTabs();
    initializeOptimizationLevel();
    initializeThemeToggle();
    initializeMobileMenu();
    loadExamples();
    
    // Button event listeners
    const compileBtn = document.getElementById('compile-btn');
    if (compileBtn) {
        compileBtn.addEventListener('click', compileCode);
    }
    const runBtn = document.getElementById('run-btn');
    if (runBtn) {
        runBtn.addEventListener('click', runCode);
    }
    const analyzeBtn = document.getElementById('analyze-btn');
    if (analyzeBtn) {
        analyzeBtn.addEventListener('click', analyzeCode);
    }
    document.getElementById('clear-btn').addEventListener('click', clearEditor);
    document.getElementById('download-source-btn').addEventListener('click', downloadSource);
    document.getElementById('download-c-btn').addEventListener('click', downloadCCode);
});