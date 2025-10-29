export const JWT_LOCAL_STORAGE_KEY = 'token';
// API Base URL from environment variable
// Leave empty in .env for production (same domain/port as backend)
// Use http://localhost:8080 in .env for development mode
export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '';