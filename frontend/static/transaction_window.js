// transaction_window.js - Scrollable transaction table
export function renderTransactionWindow(containerId, transactions) {
    const container = document.getElementById(containerId);
    if (!container) return;
    container.innerHTML = `<table class="tx-table">
      <thead><tr><th>Time</th><th>Pair</th><th>Type</th><th>Amount</th><th>Status</th></tr></thead>
      <tbody>
        ${transactions.map(tx => `<tr><td>${tx.time}</td><td>${tx.pair}</td><td>${tx.type}</td><td>${tx.amount}</td><td>${tx.status}</td></tr>`).join('')}
      </tbody>
    </table>`;
}

window.renderTransactionWindow = renderTransactionWindow;
