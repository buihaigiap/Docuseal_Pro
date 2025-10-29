import React, { useState } from 'react';
import { EditorField } from './types';
import { Plus, X } from 'lucide-react';

interface FieldPropertiesProps {
  fields: EditorField[];
  currentPartner: string;
  selectedFieldTempId: string | null;
  setSelectedFieldTempId: (id: string | null) => void;
  updateField: (tempId: string, updates: Partial<EditorField>) => void;
  getCurrentToolIcon: (fieldType: string) => React.ReactElement;
}

const FieldProperties: React.FC<FieldPropertiesProps> = ({
  fields,
  currentPartner,
  selectedFieldTempId,
  setSelectedFieldTempId,
  updateField , 
  getCurrentToolIcon
}) => {
  const [newOptionText, setNewOptionText] = useState('');

  const selectedField = fields.find(f => f.tempId === selectedFieldTempId);
  const hasOptions = selectedField && ['radio', 'multiple', 'select'].includes(selectedField.field_type);

  const handleAddOption = () => {
    if (!selectedField || !newOptionText.trim()) return;
    const currentOptions = selectedField.options || [];
    updateField(selectedField.tempId, { 
      options: [...currentOptions, newOptionText.trim()] 
    });
    setNewOptionText('');
  };

  const handleRemoveOption = (index: number) => {
    if (!selectedField) return;
    const currentOptions = selectedField.options || [];
    updateField(selectedField.tempId, { 
      options: currentOptions.filter((_, i) => i !== index) 
    });
  };

  const handleUpdateOption = (index: number, value: string) => {
    if (!selectedField) return;
    const currentOptions = [...(selectedField.options || [])];
    currentOptions[index] = value;
    updateField(selectedField.tempId, { options: currentOptions });
  };

  return (
    <div>
      <div className="space-y-4">
        <div>
          <div className="max-h-40 overflow-y-auto space-y-2">
            <div>
              <div className="space-y-1">
                {fields.filter(f => f.partner === currentPartner).map(field => (
                  <div
                    key={field.tempId}
                    className={`p-2 rounded-md text-sm ${
                      selectedFieldTempId === field.tempId
                        ? 'bg-indigo-600 '
                        : 'hover:bg-gray-600 hover:text-white '
                    }`}
                  >
                    
                     {selectedFieldTempId === field.tempId ? (
                      <div className="flex items-center text-white gap-2">
                        {getCurrentToolIcon(field.field_type)}
                        <input
                          type="text"
                          value={field.name}
                          onChange={e => updateField(field.tempId, { name: e.target.value })}
                          className="bg-transparent border-none outline-none text-white font-medium flex-1"
                        />
                      </div>
                    ) : (
                      <div
                        onClick={() => setSelectedFieldTempId(field.tempId)}
                        className="cursor-pointer font-medium flex items-center gap-2 text-white"
                      >
                         {getCurrentToolIcon(field.field_type)}
                         {field.name}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
            {fields.filter(f => !f.partner).length > 0 && (
              <div>
                <h5 className="text-sm font-medium text-gray-400 mb-1">Unassigned</h5>
                <div className="space-y-1">
                  {fields.filter(f => !f.partner).map(field => (
                    <div
                      key={field.tempId}
                      className={`p-2 rounded-md text-sm ${
                        selectedFieldTempId === field.tempId
                          ? 'bg-indigo-600 text-white'
                          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                      }`}
                    >
                      {selectedFieldTempId === field.tempId ? (
                        <div className="flex items-center">
                          {getCurrentToolIcon(field.field_type)}
                          <input
                            type="text"
                            value={field.name}
                            onChange={e => updateField(field.tempId, { name: e.target.value })}
                            className="bg-transparent border-none outline-none text-white font-medium flex-1"
                          />
                        </div>
                      ) : (
                        <div
                          onClick={() => setSelectedFieldTempId(field.tempId)}
                          className="cursor-pointer font-medium flex items-center"
                        >
                          {getCurrentToolIcon(field.field_type)}
                          <span className="ml-2">{field.name}</span>
                        </div>
                      )}
                      <div className="text-xs opacity-75 capitalize">{field.field_type}</div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Options Editor for radio, multiple, select */}
        {hasOptions && selectedField && (
          <div className="mt-4 border-t border-gray-600 pt-4">
            <h4 className="text-sm font-semibold text-white mb-2">Options</h4>
            <div className="space-y-2">
              {(selectedField.options || []).map((option, index) => (
                <div key={index} className="flex items-center gap-2">
                  <input
                    type="text"
                    value={option}
                    onChange={(e) => handleUpdateOption(index, e.target.value)}
                    className="flex-1 px-2 py-1 bg-gray-700 text-white rounded text-sm border border-gray-600 focus:border-indigo-500 outline-none"
                    placeholder={`Option ${index + 1}`}
                  />
                  <button
                    onClick={() => handleRemoveOption(index)}
                    className="p-1 text-red-400 hover:text-red-300 hover:bg-red-900/20 rounded"
                    title="Remove option"
                  >
                    <X className="w-4 h-4" />
                  </button>
                </div>
              ))}
              
              <div className="flex items-center gap-2 mt-2">
                <input
                  type="text"
                  value={newOptionText}
                  onChange={(e) => setNewOptionText(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      handleAddOption();
                    }
                  }}
                  className="flex-1 px-2 py-1 bg-gray-700 text-white rounded text-sm border border-gray-600 focus:border-indigo-500 outline-none"
                  placeholder="Add new option..."
                />
                <button
                  onClick={handleAddOption}
                  className="p-1 text-green-400 hover:text-green-300 hover:bg-green-900/20 rounded"
                  title="Add option"
                >
                  <Plus className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default FieldProperties;