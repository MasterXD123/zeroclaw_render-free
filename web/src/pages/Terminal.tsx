import { useState } from 'react';

interface ExecuteResponse {
  status: string;
  output: string;
  error?: string;
}

const quickCommands = [
  { label: 'pwd', cmd: 'pwd' },
  { label: 'ls', cmd: 'ls -la' },
  { label: 'git status', cmd: 'git status' },
  { label: 'date', cmd: 'date' },
  { label: 'whoami', cmd: 'whoami' },
  { label: 'df -h', cmd: 'df -h' },
  { label: 'free -m', cmd: 'free -m' },
  { label: 'ps aux', cmd: 'ps aux' },
];

export default function Terminal() {
  const [command, setCommand] = useState('');
  const [output, setOutput] = useState<ExecuteResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [history, setHistory] = useState<Array<{cmd: string; result: ExecuteResponse}>>([]);

  const executeCommand = async (cmd: string) => {
    if (!cmd.trim()) return;
    
    setLoading(true);
    setCommand('');
    
    try {
      const response = await fetch('/api/execute', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ command: cmd }),
      });
      
      const result = await response.json() as ExecuteResponse;
      setOutput(result);
      setHistory(prev => [...prev, { cmd, result }]);
    } catch (err) {
      setOutput({
        status: 'error',
        output: '',
        error: `Request failed: ${err}`
      });
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    executeCommand(command);
  };

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">Terminal</h1>
      
      <div className="mb-4">
        <p className="text-sm text-gray-400 mb-2">Quick Commands:</p>
        <div className="flex flex-wrap gap-2">
          {quickCommands.map((item) => (
            <button
              key={item.label}
              onClick={() => executeCommand(item.cmd)}
              disabled={loading}
              className="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded transition-colors disabled:opacity-50"
            >
              {item.label}
            </button>
          ))}
        </div>
      </div>

      <form onSubmit={handleSubmit} className="mb-4">
        <div className="flex gap-2">
          <input
            type="text"
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            placeholder="Enter command..."
            className="flex-1 px-4 py-2 bg-gray-800 border border-gray-700 rounded text-white placeholder-gray-500 focus:outline-none focus:border-blue-500"
            disabled={loading}
          />
          <button
            type="submit"
            disabled={loading || !command.trim()}
            className="px-6 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? 'Running...' : 'Execute'}
          </button>
        </div>
      </form>

      {output && (
        <div className="mb-4 p-4 bg-gray-900 rounded border border-gray-700">
          <div className="flex items-center gap-2 mb-2">
            <span className={`w-2 h-2 rounded-full ${output.status === 'ok' ? 'bg-green-500' : 'bg-red-500'}`}></span>
            <span className="text-sm text-gray-400">Status: {output.status}</span>
          </div>
          {output.error && (
            <div className="mb-2 p-2 bg-red-900/30 border border-red-800 rounded text-red-400 text-sm">
              Error: {output.error}
            </div>
          )}
          {output.output && (
            <pre className="p-2 bg-black rounded text-green-400 text-sm overflow-x-auto whitespace-pre-wrap">
{output.output}
            </pre>
          )}
        </div>
      )}

      {history.length > 0 && (
        <div>
          <h2 className="text-lg font-semibold mb-2">History</h2>
          <div className="space-y-2">
            {history.slice().reverse().map((item, idx) => (
              <div key={idx} className="p-3 bg-gray-800 rounded">
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-blue-400">$</span>
                  <span className="text-white">{item.cmd}</span>
                </div>
                {item.result.output && (
                  <pre className="p-2 bg-black rounded text-green-400 text-xs overflow-x-auto whitespace-pre-wrap max-h-32">
{item.result.output}
                  </pre>
                )}
                {item.result.error && (
                  <div className="p-2 bg-red-900/30 border border-red-800 rounded text-red-400 text-xs">
                    {item.result.error}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}