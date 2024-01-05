INSTALL_DIR := \
	$(shell \
		if (echo $(PATH) | tr ':' '\n' \
				| grep -q "^$(HOME)/[.]local/bin$$"); then \
			echo "$(HOME)/.local/bin"; \
		elif (echo $(PATH) | tr ':' '\n' \
				| grep -q "^$(HOME)/bin$$"); then \
			echo "$(HOME)/bin"; \
		else \
			echo "NOT_FOUND"; \
		fi)

.PHONY: build
build:

.PHONY: install
install:
	[ "$(shell which perl > /dev/null; echo $$?)" = "0" ] \
		|| (echo "No perl installation found."; exit 1)
	[ "$(shell which objdump > /dev/null; echo $$?)" = "0" ] \
		|| (echo "No objdump installation found."; exit 1)
	[ "$(INSTALL_DIR)" != "NOT_FOUND" ] \
		|| (echo "No user-specific bin directory in PATH."; exit 1)
	cp lsx86features.pl $(INSTALL_DIR)/lsx86features
	chmod +x $(INSTALL_DIR)/lsx86features

.PHONY: uninstall
uninstall:
	[ "$(INSTALL_DIR)" != "NOT_FOUND" ] \
		|| (echo "No user-specific bin directory in PATH."; exit 1)
	rm $(INSTALL_DIR)/lsx86features
