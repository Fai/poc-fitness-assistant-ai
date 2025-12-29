#!/bin/bash
set -e

echo "🚀 Setting up Fitness Assistant AI development environment..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Start development services
echo "📦 Starting PostgreSQL and Redis..."
docker-compose -f docker-compose.dev.yml up -d

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
until docker exec fitness-assistant-postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 1
done
echo "✅ PostgreSQL is ready"

# Wait for Redis to be ready
echo "⏳ Waiting for Redis to be ready..."
until docker exec fitness-assistant-redis redis-cli ping > /dev/null 2>&1; do
    sleep 1
done
echo "✅ Redis is ready"

# Run database migrations
echo "🔄 Running database migrations..."
sqlx migrate run --source backend/migrations

echo ""
echo "✅ Development environment is ready!"
echo ""
echo "Services:"
echo "  - PostgreSQL: localhost:5432"
echo "  - Redis: localhost:6379"
echo "  - Adminer (DB UI): http://localhost:8081"
echo "  - Redis Commander: http://localhost:8082"
echo ""
echo "To start the backend server:"
echo "  cargo run --bin fitness-assistant-backend"
