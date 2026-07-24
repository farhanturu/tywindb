/**
 * Tywindb GUI — Database Manager
 * 
 * A modern, user-friendly interface for managing Tywindb databases.
 */

class TywindbGUI {
    constructor() {
        this.db = null;
        this.currentTable = null;
        this.queryHistory = [];
        
        this.init();
    }
    
    init() {
        this.bindEvents();
        this.loadTables();
    }
    
    // ============================================
    // Event Binding
    // ============================================
    
    bindEvents() {
        // Query Editor
        const editor = document.getElementById('query-editor');
        const runBtn = document.getElementById('btn-run');
        const clearBtn = document.getElementById('btn-clear');
        const formatBtn = document.getElementById('btn-format');
        
        runBtn.addEventListener('click', () => this.runQuery());
        clearBtn.addEventListener('click', () => this.clearEditor());
        formatBtn.addEventListener('click', () => this.formatQuery());
        
        // Keyboard shortcuts
        editor.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'Enter') {
                e.preventDefault();
                this.runQuery();
            }
        });
        
        // Sidebar
        document.getElementById('refresh-tables').addEventListener('click', () => this.loadTables());
        document.getElementById('btn-new-table').addEventListener('click', () => this.showModal('modal-create-table'));
        
        // Results
        document.getElementById('btn-copy-results').addEventListener('click', () => this.copyResults());
        document.getElementById('btn-export-results').addEventListener('click', () => this.exportResults());
        
        // Help
        document.getElementById('btn-help').addEventListener('click', () => this.showModal('modal-help'));
        
        // Modal close buttons
        document.querySelectorAll('.modal-close, .modal-cancel').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const modal = e.target.closest('.modal');
                if (modal) this.hideModal(modal.id);
            });
        });
        
        // Create table
        document.getElementById('btn-add-column').addEventListener('click', () => this.addColumn());
        document.getElementById('btn-create-table').addEventListener('click', () => this.createTable());
        
        // Modal backdrop click
        document.querySelectorAll('.modal').forEach(modal => {
            modal.addEventListener('click', (e) => {
                if (e.target === modal) {
                    this.hideModal(modal.id);
                }
            });
        });
    }
    
    // ============================================
    // Query Execution
    // ============================================
    
    async runQuery() {
        const editor = document.getElementById('query-editor');
        const sql = editor.value.trim();
        
        if (!sql) {
            this.showToast('Please enter a query', 'error');
            return;
        }
        
        const startTime = performance.now();
        
        try {
            // Send query to server
            const response = await fetch('/api/query', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query: sql })
            });
            
            const result = await response.json();
            const endTime = performance.now();
            const executionTime = Math.round(endTime - startTime);
            
            if (result.error) {
                this.showToast(result.error, 'error');
                this.showEmptyResults();
            } else {
                this.displayResults(result.data, result.rows_affected);
                this.updateExecutionTime(executionTime);
                this.showToast('Query executed successfully', 'success');
            }
            
            // Update last query
            document.getElementById('last-query').textContent = `Last query: ${sql.substring(0, 30)}...`;
            
            // Add to history
            this.queryHistory.push({
                query: sql,
                timestamp: new Date(),
                success: !result.error
            });
            
        } catch (error) {
            this.showToast('Failed to execute query: ' + error.message, 'error');
            this.showEmptyResults();
        }
    }
    
    // ============================================
    // Results Display
    // ============================================
    
    displayResults(data, rowsAffected) {
        const container = document.getElementById('results-table');
        const countEl = document.getElementById('result-count');
        
        if (!data || data.length === 0) {
            if (rowsAffected !== undefined) {
                container.innerHTML = `<div class="empty-state"><p>Query OK, ${rowsAffected} rows affected</p></div>`;
                countEl.textContent = `${rowsAffected} rows affected`;
            } else {
                this.showEmptyResults();
            }
            return;
        }
        
        // Build table
        const columns = Object.keys(data[0]);
        let html = '<table><thead><tr>';
        
        // Headers
        columns.forEach(col => {
            html += `<th>${this.escapeHtml(col)}</th>`;
        });
        html += '</tr></thead><tbody>';
        
        // Rows
        data.forEach(row => {
            html += '<tr>';
            columns.forEach(col => {
                const value = row[col];
                const displayValue = value === null ? '<span class="null">NULL</span>' : this.escapeHtml(String(value));
                html += `<td>${displayValue}</td>`;
            });
            html += '</tr>';
        });
        
        html += '</tbody></table>';
        container.innerHTML = html;
        
        // Update count
        countEl.textContent = `${data.length} rows`;
    }
    
    showEmptyResults() {
        const container = document.getElementById('results-table');
        container.innerHTML = '<div class="empty-state"><p>Run a query to see results</p></div>';
        document.getElementById('result-count').textContent = '0 rows';
        document.getElementById('execution-time').textContent = '0ms';
    }
    
    updateExecutionTime(ms) {
        document.getElementById('execution-time').textContent = `${ms}ms`;
    }
    
    // ============================================
    // Table Management
    // ============================================
    
    async loadTables() {
        const list = document.getElementById('table-list');
        list.innerHTML = '<li class="loading">Loading...</li>';
        
        try {
            const response = await fetch('/api/tables');
            const result = await response.json();
            
            if (result.tables && result.tables.length > 0) {
                list.innerHTML = result.tables.map(table => 
                    `<li data-table="${table}">${table}</li>`
                ).join('');
                
                // Add click handlers
                list.querySelectorAll('li[data-table]').forEach(li => {
                    li.addEventListener('click', () => {
                        this.selectTable(li.dataset.table);
                    });
                });
                
                document.getElementById('table-count').textContent = `${result.tables.length} tables`;
            } else {
                list.innerHTML = '<li class="loading">No tables yet</li>';
                document.getElementById('table-count').textContent = '0 tables';
            }
        } catch (error) {
            list.innerHTML = '<li class="loading">Error loading tables</li>';
            console.error('Failed to load tables:', error);
        }
    }
    
    selectTable(tableName) {
        this.currentTable = tableName;
        
        // Update UI
        document.querySelectorAll('.table-list li').forEach(li => {
            li.classList.toggle('active', li.dataset.table === tableName);
        });
        
        // Load table data
        const editor = document.getElementById('query-editor');
        editor.value = `SELECT * FROM ${tableName};`;
        this.runQuery();
    }
    
    // ============================================
    // Create Table
    // ============================================
    
    addColumn() {
        const list = document.getElementById('columns-list');
        const row = document.createElement('div');
        row.className = 'column-row';
        row.innerHTML = `
            <input type="text" placeholder="Column name" class="col-name">
            <select class="col-type">
                <option value="INTEGER">INTEGER</option>
                <option value="TEXT">TEXT</option>
                <option value="FLOAT">FLOAT</option>
                <option value="BOOLEAN">BOOLEAN</option>
                <option value="BLOB">BLOB</option>
            </select>
            <label><input type="checkbox" class="col-primary"> PK</label>
            <label><input type="checkbox" class="col-notnull" checked> NN</label>
            <button class="btn btn-icon btn-danger remove-col">&times;</button>
        `;
        
        row.querySelector('.remove-col').addEventListener('click', () => row.remove());
        list.appendChild(row);
    }
    
    async createTable() {
        const name = document.getElementById('table-name').value.trim();
        const columns = [];
        
        document.querySelectorAll('.column-row').forEach(row => {
            const colName = row.querySelector('.col-name').value.trim();
            const colType = row.querySelector('.col-type').value;
            
            if (colName) {
                columns.push({ name: colName, type: colType });
            }
        });
        
        if (!name) {
            this.showToast('Please enter a table name', 'error');
            return;
        }
        
        if (columns.length === 0) {
            this.showToast('Please add at least one column', 'error');
            return;
        }
        
        // Build SQL
        const colDefs = columns.map(c => `${c.name} ${c.type}`).join(', ');
        const sql = `CREATE TABLE ${name} (${colDefs});`;
        
        try {
            const response = await fetch('/api/query', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query: sql })
            });
            
            const result = await response.json();
            
            if (result.error) {
                this.showToast('Failed to create table: ' + result.error, 'error');
            } else {
                this.showToast('Table created successfully', 'success');
                this.hideModal('modal-create-table');
                this.loadTables();
                
                // Clear form
                document.getElementById('table-name').value = '';
                document.getElementById('columns-list').innerHTML = '';
                this.addColumn();
            }
        } catch (error) {
            this.showToast('Failed to create table: ' + error.message, 'error');
        }
    }
    
    // ============================================
    // Utility Functions
    // ============================================
    
    clearEditor() {
        document.getElementById('query-editor').value = '';
    }
    
    formatQuery() {
        const editor = document.getElementById('query-editor');
        let sql = editor.value;
        
        // Simple SQL formatting
        const keywords = ['SELECT', 'FROM', 'WHERE', 'AND', 'OR', 'INSERT', 'INTO', 'VALUES', 
                         'UPDATE', 'SET', 'DELETE', 'CREATE', 'TABLE', 'BEGIN', 'COMMIT', 'ROLLBACK'];
        
        keywords.forEach(keyword => {
            const regex = new RegExp(`\\b${keyword}\\b`, 'gi');
            sql = sql.replace(regex, keyword);
        });
        
        editor.value = sql;
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    // ============================================
    // Clipboard & Export
    // ============================================
    
    async copyResults() {
        const table = document.querySelector('.results-table table');
        if (!table) {
            this.showToast('No results to copy', 'error');
            return;
        }
        
        // Convert table to CSV
        let csv = '';
        const rows = table.querySelectorAll('tr');
        
        rows.forEach(row => {
            const cells = row.querySelectorAll('th, td');
            const rowData = Array.from(cells).map(cell => cell.textContent);
            csv += rowData.join('\t') + '\n';
        });
        
        try {
            await navigator.clipboard.writeText(csv);
            this.showToast('Results copied to clipboard', 'success');
        } catch (error) {
            this.showToast('Failed to copy results', 'error');
        }
    }
    
    exportResults() {
        const table = document.querySelector('.results-table table');
        if (!table) {
            this.showToast('No results to export', 'error');
            return;
        }
        
        // Convert table to CSV
        let csv = '';
        const rows = table.querySelectorAll('tr');
        
        rows.forEach(row => {
            const cells = row.querySelectorAll('th, td');
            const rowData = Array.from(cells).map(cell => {
                const value = cell.textContent;
                // Escape CSV values
                if (value.includes(',') || value.includes('"') || value.includes('\n')) {
                    return `"${value.replace(/"/g, '""')}"`;
                }
                return value;
            });
            csv += rowData.join(',') + '\n';
        });
        
        // Download CSV
        const blob = new Blob([csv], { type: 'text/csv' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `tywindb_export_${Date.now()}.csv`;
        a.click();
        URL.revokeObjectURL(url);
        
        this.showToast('Results exported as CSV', 'success');
    }
    
    // ============================================
    // Modal Management
    // ============================================
    
    showModal(modalId) {
        document.getElementById(modalId).classList.remove('hidden');
    }
    
    hideModal(modalId) {
        document.getElementById(modalId).classList.add('hidden');
    }
    
    // ============================================
    // Toast Notifications
    // ============================================
    
    showToast(message, type = 'info') {
        const container = document.getElementById('toast-container');
        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.textContent = message;
        container.appendChild(toast);
        
        setTimeout(() => {
            toast.remove();
        }, 3000);
    }
}

// Initialize GUI
document.addEventListener('DOMContentLoaded', () => {
    window.tywindb = new TywindbGUI();
});
