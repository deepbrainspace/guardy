.PHONY: bench-setup bench-prepare bench bench-clean

# Install all required benchmarking tools
bench-prepare:
	@echo "📦 Installing benchmarking dependencies..."

	# Install gitleaks for comparison
	@if ! command -v gitleaks &> /dev/null; then \
		echo "🔧 Installing gitleaks..."; \
		if command -v brew &> /dev/null; then \
			brew install gitleaks; \
		elif command -v apt &> /dev/null; then \
			wget -O gitleaks.deb https://github.com/gitleaks/gitleaks/releases/download/v8.21.2/gitleaks_8.21.2_linux_x64.deb && \
			sudo dpkg -i gitleaks.deb && \
			rm gitleaks.deb; \
		else \
			echo "❌ Cannot install gitleaks automatically"; \
		fi \
	else \
		echo "✅ gitleaks already installed"; \
	fi

	# Install gnuplot for better criterion plots
	@if ! command -v gnuplot &> /dev/null; then \
		echo "🔧 Installing gnuplot..."; \
		if command -v apt &> /dev/null; then \
			sudo apt update && sudo apt install -y gnuplot; \
		else \
			echo "⚠️  Cannot install gnuplot automatically"; \
		fi \
	else \
		echo "✅ gnuplot already installed"; \
	fi

	@echo "✅ Benchmark preparation complete!"

# Setup test data for benchmarks
bench-setup: bench-prepare
	@echo "🧪 Setting up benchmark test data..."
	@if [ ! -d "benches/test-data" ]; then \
		mkdir -p benches/test-data && \
		cd benches && \
		echo "Generating test data..." && \
		bash -c "cd ../../../.. && find . -name '*.rs' -o -name '*.toml' -o -name '*.md' | head -1000 | xargs -I {} cp {} /tmp/bench-{}" 2>/dev/null || true; \
		for size in small medium large; do \
			mkdir -p test-data/$$size-project; \
			git init test-data/$$size-project --quiet; \
			cd test-data/$$size-project; \
			for i in $$(seq 1 100); do \
				echo "// Test file $$i" > "file$$i.rs"; \
				echo "test content $$i" > "content$$i.txt"; \
			done; \
			if [ "$$size" = "large" ]; then \
				for i in $$(seq 101 1000); do \
					echo "// Large test file $$i" > "large$$i.rs"; \
				done; \
			fi; \
			echo 'api_key = "sk-1234567890abcdef"' > config.toml; \
			git add . && git commit -m "Initial commit" --quiet; \
			cd ../..; \
		done; \
	else \
		echo "✅ Test data already exists"; \
	fi

# Run benchmarks with proper setup
bench: bench-setup
	@echo "🚀 Running Guardy benchmarks..."
	cargo bench

# Clean benchmark artifacts
bench-clean:
	@echo "🧹 Cleaning benchmark artifacts..."
	rm -rf benches/test-data target/criterion
	@echo "✅ Benchmark cleanup complete!"
