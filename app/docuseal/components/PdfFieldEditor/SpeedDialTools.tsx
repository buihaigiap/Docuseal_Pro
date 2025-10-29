import React, { useState, useEffect } from 'react';
import { FieldType } from '../../types';
import { FieldTool } from './types';
import { fieldTools } from './constants';

interface SpeedDialToolsProps {
  activeTool: FieldTool;
  setActiveTool: (tool: FieldTool) => void;
  setLastFieldTool: (tool: FieldType) => void;
  onAddField?: (fieldType: FieldTool) => void;
}

const SpeedDialTools: React.FC<SpeedDialToolsProps> = ({ activeTool, setActiveTool, setLastFieldTool, onAddField }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [positionAbove, setPositionAbove] = useState(false);

  useEffect(() => {
    const checkPosition = () => {
      // Button is 48px from bottom, actions need about 120px height
      // If viewport height is less than 48 + 120 + some padding = ~200px, position above
      setPositionAbove(window.innerHeight < 200);
    };

    checkPosition();
    window.addEventListener('resize', checkPosition);
    return () => window.removeEventListener('resize', checkPosition);
  }, []);

  const handleToolClick = (tool: typeof fieldTools[0]) => {
    setActiveTool(tool.type);
    if (tool.type !== 'cursor') setLastFieldTool(tool.type as any);
    setIsOpen(false);
    
    // If onAddField is provided and tool is not cursor, call it
    if (onAddField && tool.type !== 'cursor') {
      onAddField(tool.type);
    }
  };

  return (
    <div className="fixed bottom-4 right-4 z-[9999] touch-none">
      {/* Speed Dial Actions */}
      {isOpen && (
        <div className={`absolute right-0 mb-2 z-[10000] ${positionAbove ? 'top-0 transform -translate-y-full' : 'bottom-14'}`}>
          <div className="bg-gray-800 rounded-lg p-2 shadow-lg border border-gray-600">
            <div className="grid grid-cols-3 gap-1 min-w-max">
              {fieldTools.map((tool, index) => (
                <button
                  key={tool.name}
                  onClick={() => handleToolClick(tool)}
                  onTouchStart={(e) => {
                    e.preventDefault();
                    handleToolClick(tool);
                  }}
                  className={`flex flex-col items-center justify-center p-2 rounded-md transition-all duration-300 text-xs space-y-1 w-16 h-16 transform hover:scale-105 active:scale-95 touch-manipulation
                    ${activeTool === tool.type ? 'bg-indigo-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'}`}
                  title={tool.type === 'cursor' ? 'Select Field' : `Add ${tool.name} Field`}
                  style={{
                    animationDelay: `${index * 50}ms`,
                    animation: 'fadeInUp 0.3s ease-out forwards',
                    minWidth: '64px',
                    minHeight: '64px'
                  }}
                >
                  {tool.iconComponent('w-6 h-6')}
                  <span className="text-center leading-tight text-xs">{tool.name}</span>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      <button
        onClick={(e) => {
          e.preventDefault();
          setIsOpen(!isOpen);
        }}
        className={`w-12 h-12 rounded-full flex items-center justify-center transition-all duration-300 shadow-lg touch-manipulation ${
          isOpen ? 'bg-indigo-600 rotate-45' : 'bg-gray-700 hover:bg-gray-600'
        }`}
        title="Tools"
        style={{
          minWidth: '48px',
          minHeight: '48px'
        }}
      >
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" className="w-5 h-5 text-white">
          <path d="M19 13H13V19H11V13H5V11H11V5H13V11H19V13Z" fill="currentColor"/>
        </svg>
      </button>

      {/* Overlay removed - only close when clicking main button */}

      <style dangerouslySetInnerHTML={{
        __html: `
          @keyframes fadeInUp {
            from {
              opacity: 0;
              transform: translateY(10px) scale(0.9);
            }
            to {
              opacity: 1;
              transform: translateY(0) scale(1);
            }
          }
        `
      }} />
    </div>
  );
};

export default SpeedDialTools;