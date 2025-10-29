import React from 'react';
import { FieldTool } from './types';
import { fieldTools } from './constants';

interface FieldToolsProps {
  activeTool: FieldTool;
  setActiveTool: (tool: FieldTool) => void;
  setLastFieldTool: (tool: any) => void;
}

const FieldTools: React.FC<FieldToolsProps> = ({ activeTool, setActiveTool, setLastFieldTool }) => {
  return (
    <div>
      <h3 className="text-xl font-semibold mb-4 pb-2 text-white">Tools</h3>
      <div className="grid grid-cols-3 gap-2">
        {fieldTools.map(tool => (
          <button
            key={tool.name}
            onClick={() => {
              setActiveTool(tool.type);
              if (tool.type !== 'cursor') setLastFieldTool(tool.type as any);
            }}
            className={`flex flex-col items-center text-white justify-center p-2 rounded-md transition-colors text-xs space-y-1 h-20
              ${activeTool === tool.type ? 'bg-indigo-600' : 'hover:bg-gray-600 hover:text-white'}`}
            title={tool.type === 'cursor' ? 'Select Field' : `Add ${tool.name} Field`}
          >
            {tool.iconComponent(activeTool === tool.type ? 'w-6 h-6 text-white' : 'w-6 h-6')}
            <span>{tool.name}</span>
          </button>
        ))}
      </div>
    </div>
  );
};

export default FieldTools;