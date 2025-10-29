#!/bin/bash

# Development mode script - runs FE and BE separately with hot reload

echo "🔧 Development Mode Setup"
echo ""

# Check backend .env
if [ ! -f .env ]; then
    echo "❌ Backend .env not found! Please create it first."
    exit 1
fi

# Setup frontend .env for development
if [ ! -f app/docuseal/.env ]; then
    echo "📝 Creating frontend .env for development..."
    cat > app/docuseal/.env << 'EOF'
# Development Mode Configuration
# Backend runs on port 8080, Frontend on port 3000
VITE_API_BASE_URL=http://localhost:8080

# Gemini API Key (optional)
# GEMINI_API_KEY=
EOF
    echo "✅ Created app/docuseal/.env"
else
    echo "✅ Frontend .env exists"
fi

echo ""
echo "📋 Development Mode Instructions:"
echo ""
echo "1️⃣  Terminal 1 - Start Backend (port 8080):"
echo "   cargo run"
echo ""
echo "2️⃣  Terminal 2 - Start Frontend (port 3000):"
echo "   cd app/docuseal && npm run dev"
echo ""
echo "3️⃣  Access:"
echo "   Frontend: http://localhost:3000 (with hot reload)"
echo "   Backend API: http://localhost:8080/api"
echo "   Swagger: http://localhost:8080/swagger-ui"
echo ""
echo "Press any key to start backend now, or Ctrl+C to cancel..."
read -n 1 -s

echo ""
echo "🚀 Starting backend..."
cargo run
