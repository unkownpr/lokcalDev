<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>LokcalDev</title>
<style>
  *, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

  body {
    font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: hsl(0 0% 98%);
    color: hsl(0 0% 10%);
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  .header {
    background: hsl(0 0% 100%);
    border-bottom: 1px solid hsl(0 0% 91%);
    padding: 0 2rem;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .header h1 {
    font-size: 14px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .header-nav {
    display: flex;
    gap: 0.5rem;
  }

  .header-nav a {
    font-size: 12px;
    font-weight: 500;
    color: hsl(0 0% 45%);
    text-decoration: none;
    padding: 6px 12px;
    border-radius: 6px;
    transition: all 0.15s;
  }

  .header-nav a:hover {
    background: hsl(0 0% 96%);
    color: hsl(0 0% 12%);
  }

  .container {
    max-width: 1100px;
    margin: 0 auto;
    padding: 2rem;
  }

  .hero {
    text-align: center;
    padding: 3rem 0 2.5rem;
  }

  .hero-logo {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    background: hsl(0 0% 12%);
    border-radius: 12px;
    margin-bottom: 1rem;
  }

  .hero-logo svg {
    width: 24px;
    height: 24px;
    color: hsl(0 0% 98%);
  }

  .hero h2 {
    font-size: 24px;
    font-weight: 600;
    letter-spacing: -0.025em;
    margin-bottom: 0.5rem;
  }

  .hero p {
    font-size: 13px;
    color: hsl(0 0% 45%);
    max-width: 400px;
    margin: 0 auto;
    line-height: 1.5;
  }

  .cards {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .card {
    background: hsl(0 0% 100%);
    border: 1px solid hsl(0 0% 91%);
    border-radius: 8px;
    padding: 1rem;
  }

  .card-label {
    font-size: 11px;
    font-weight: 500;
    color: hsl(0 0% 45%);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.25rem;
  }

  .card-value {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.025em;
  }

  .card-desc {
    font-size: 11px;
    color: hsl(0 0% 55%);
    margin-top: 0.25rem;
  }

  .status-dot {
    display: inline-block;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    margin-right: 6px;
    position: relative;
    top: -1px;
  }

  .status-dot.green { background: hsl(142 71% 45%); }
  .status-dot.red { background: hsl(0 84% 60%); }
  .status-dot.gray { background: hsl(0 0% 75%); }

  .section {
    margin-bottom: 2rem;
  }

  .section-title {
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 0.75rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .section-title .icon {
    width: 16px;
    height: 16px;
    color: hsl(0 0% 45%);
  }

  .grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .table-card {
    background: hsl(0 0% 100%);
    border: 1px solid hsl(0 0% 91%);
    border-radius: 8px;
    overflow: hidden;
  }

  .table-card-header {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid hsl(0 0% 91%);
    font-size: 12px;
    font-weight: 600;
  }

  .table-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    font-size: 12px;
    border-bottom: 1px solid hsl(0 0% 96%);
  }

  .table-row:last-child { border-bottom: none; }

  .table-row .label {
    color: hsl(0 0% 45%);
  }

  .table-row .value {
    font-weight: 500;
    font-family: "SF Mono", "Cascadia Code", "Fira Code", monospace;
    font-size: 11px;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    font-size: 10px;
    font-weight: 500;
    padding: 2px 8px;
    border-radius: 9999px;
    border: 1px solid hsl(0 0% 91%);
    background: hsl(0 0% 96%);
    color: hsl(0 0% 35%);
  }

  .badge.green {
    background: hsl(142 76% 94%);
    border-color: hsl(142 71% 80%);
    color: hsl(142 71% 30%);
  }

  .quick-links {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
    margin-bottom: 2rem;
  }

  .quick-link {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: hsl(0 0% 100%);
    border: 1px solid hsl(0 0% 91%);
    border-radius: 8px;
    padding: 0.875rem 1rem;
    text-decoration: none;
    color: inherit;
    transition: all 0.15s;
  }

  .quick-link:hover {
    border-color: hsl(0 0% 80%);
    box-shadow: 0 1px 3px rgba(0,0,0,0.04);
  }

  .quick-link .ql-icon {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    background: hsl(0 0% 96%);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .quick-link .ql-icon svg {
    width: 16px;
    height: 16px;
    color: hsl(0 0% 35%);
  }

  .quick-link .ql-text h4 {
    font-size: 12px;
    font-weight: 600;
    margin-bottom: 1px;
  }

  .quick-link .ql-text p {
    font-size: 11px;
    color: hsl(0 0% 45%);
  }

  .phpinfo-section {
    background: hsl(0 0% 100%);
    border: 1px solid hsl(0 0% 91%);
    border-radius: 8px;
    overflow: hidden;
    margin-bottom: 2rem;
  }

  .phpinfo-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.75rem 1rem;
    font-size: 12px;
    font-weight: 600;
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
    font-family: inherit;
  }

  .phpinfo-toggle:hover { background: hsl(0 0% 98%); }

  .phpinfo-toggle .arrow {
    transition: transform 0.2s;
    color: hsl(0 0% 45%);
  }

  .phpinfo-content {
    border-top: 1px solid hsl(0 0% 91%);
  }

  /* Override phpinfo() styles to match our design */
  .phpinfo-content table {
    width: 100%;
    border-collapse: collapse;
  }

  .phpinfo-content td, .phpinfo-content th {
    font-size: 11px;
    padding: 4px 12px;
    border-bottom: 1px solid hsl(0 0% 96%);
    font-family: "SF Mono", "Cascadia Code", "Fira Code", monospace;
    vertical-align: top;
  }

  .phpinfo-content th {
    background: hsl(0 0% 98%);
    text-align: left;
    font-weight: 600;
    font-family: "Inter", -apple-system, sans-serif;
    color: hsl(0 0% 35%);
  }

  .phpinfo-content td.e {
    width: 35%;
    color: hsl(0 0% 35%);
    background: hsl(0 0% 99%);
  }

  .phpinfo-content td.v {
    word-break: break-all;
  }

  .phpinfo-content h1, .phpinfo-content h2 {
    font-family: "Inter", -apple-system, sans-serif;
    font-size: 13px;
    font-weight: 600;
    padding: 0.625rem 1rem;
    background: hsl(0 0% 97%);
    border-bottom: 1px solid hsl(0 0% 91%);
    margin: 0;
  }

  .phpinfo-content h1 { display: none; }

  .phpinfo-content img { display: none; }

  .phpinfo-content hr { display: none; }

  .phpinfo-content .p { font-size: 11px; padding: 6px 12px; }

  .footer {
    text-align: center;
    padding: 1.5rem 0 2rem;
    font-size: 11px;
    color: hsl(0 0% 65%);
  }

  .footer a {
    color: hsl(0 0% 45%);
    text-decoration: none;
  }

  .footer a:hover { text-decoration: underline; }

  @media (max-width: 768px) {
    .cards { grid-template-columns: repeat(2, 1fr); }
    .grid-2 { grid-template-columns: 1fr; }
    .quick-links { grid-template-columns: 1fr; }
  }
</style>
</head>
<body>

<?php
$php_version = phpversion();
$nginx_version = $_SERVER['SERVER_SOFTWARE'] ?? 'Nginx';
$os = php_uname('s') . ' ' . php_uname('r');
$server_name = $_SERVER['SERVER_NAME'] ?? 'localhost';
$doc_root = $_SERVER['DOCUMENT_ROOT'] ?? '-';
$loaded_extensions = get_loaded_extensions();
sort($loaded_extensions);
$ext_count = count($loaded_extensions);
$memory_limit = ini_get('memory_limit');
$max_upload = ini_get('upload_max_filesize');
$max_exec = ini_get('max_execution_time');
$display_errors = ini_get('display_errors');
?>

<div class="header">
  <h1>LokcalDev</h1>
  <div class="header-nav">
    <a href="/phpmyadmin" target="_blank">phpMyAdmin</a>
    <a href="?phpinfo=1">PHP Info</a>
  </div>
</div>

<div class="container">
  <div class="hero">
    <div class="hero-logo">
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 0 1-3-3m3 3a3 3 0 1 0 0 6h13.5a3 3 0 1 0 0-6m-16.5-3a3 3 0 0 1 3-3h13.5a3 3 0 0 1 3 3m-19.5 0a4.5 4.5 0 0 1 .9-2.7L5.737 5.1a3.375 3.375 0 0 1 2.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 0 1 .9 2.7m0 0a3 3 0 0 1-3 3m0 3h.008v.008h-.008v-.008Zm0-6h.008v.008h-.008v-.008Zm-3 6h.008v.008h-.008v-.008Zm0-6h.008v.008h-.008v-.008Z" />
      </svg>
    </div>
    <h2>Your local dev environment is running</h2>
    <p>LokcalDev is serving this page. Manage your sites, services, and databases from the desktop app.</p>
  </div>

  <!-- Summary Cards -->
  <div class="cards">
    <div class="card">
      <div class="card-label">PHP Version</div>
      <div class="card-value"><?= $php_version ?></div>
      <div class="card-desc"><span class="status-dot green"></span>FPM Active</div>
    </div>
    <div class="card">
      <div class="card-label">Web Server</div>
      <div class="card-value"><?= htmlspecialchars($nginx_version) ?></div>
      <div class="card-desc"><span class="status-dot green"></span>Listening on :80</div>
    </div>
    <div class="card">
      <div class="card-label">Extensions</div>
      <div class="card-value"><?= $ext_count ?></div>
      <div class="card-desc">Loaded modules</div>
    </div>
    <div class="card">
      <div class="card-label">Platform</div>
      <div class="card-value"><?= PHP_OS ?></div>
      <div class="card-desc"><?= php_uname('m') ?></div>
    </div>
  </div>

  <!-- Quick Links -->
  <div class="section">
    <div class="section-title">
      <svg class="icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M13.19 8.688a4.5 4.5 0 0 1 1.242 7.244l-4.5 4.5a4.5 4.5 0 0 1-6.364-6.364l1.757-1.757m9.193-9.193a4.5 4.5 0 0 1 6.364 6.364l-4.5 4.5a4.5 4.5 0 0 1-7.244-1.242" /></svg>
      Quick Links
    </div>
    <div class="quick-links">
      <a href="/phpmyadmin" class="quick-link" target="_blank">
        <div class="ql-icon">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125" /></svg>
        </div>
        <div class="ql-text">
          <h4>phpMyAdmin</h4>
          <p>Manage your databases</p>
        </div>
      </a>
      <a href="?phpinfo=1" class="quick-link">
        <div class="ql-icon">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" /></svg>
        </div>
        <div class="ql-text">
          <h4>PHP Info</h4>
          <p>Full configuration details</p>
        </div>
      </a>
      <a href="mailto:" class="quick-link" onclick="event.preventDefault();">
        <div class="ql-icon">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M12 6.042A8.967 8.967 0 0 0 6 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 0 1 6 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 0 1 6-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0 0 18 18a8.967 8.967 0 0 0-6 2.292m0-14.25v14.25" /></svg>
        </div>
        <div class="ql-text">
          <h4>Documentation</h4>
          <p>LokcalDev guides</p>
        </div>
      </a>
    </div>
  </div>

  <!-- Environment Details -->
  <div class="section">
    <div class="section-title">
      <svg class="icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z" /><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" /></svg>
      Environment
    </div>
    <div class="grid-2">
      <div class="table-card">
        <div class="table-card-header">PHP Configuration</div>
        <div class="table-row"><span class="label">Version</span><span class="value"><?= $php_version ?></span></div>
        <div class="table-row"><span class="label">SAPI</span><span class="value"><?= php_sapi_name() ?></span></div>
        <div class="table-row"><span class="label">Memory Limit</span><span class="value"><?= $memory_limit ?></span></div>
        <div class="table-row"><span class="label">Upload Max</span><span class="value"><?= $max_upload ?></span></div>
        <div class="table-row"><span class="label">Max Execution</span><span class="value"><?= $max_exec ?>s</span></div>
        <div class="table-row"><span class="label">Display Errors</span><span class="value"><?= $display_errors ? 'On' : 'Off' ?></span></div>
        <div class="table-row"><span class="label">Timezone</span><span class="value"><?= ini_get('date.timezone') ?: 'default' ?></span></div>
        <div class="table-row"><span class="label">OPcache</span><span class="value"><?= function_exists('opcache_get_status') ? '<span class="badge green">Enabled</span>' : '<span class="badge">Disabled</span>' ?></span></div>
      </div>
      <div class="table-card">
        <div class="table-card-header">Server</div>
        <div class="table-row"><span class="label">Web Server</span><span class="value"><?= htmlspecialchars($nginx_version) ?></span></div>
        <div class="table-row"><span class="label">Hostname</span><span class="value"><?= htmlspecialchars($server_name) ?></span></div>
        <div class="table-row"><span class="label">Document Root</span><span class="value" title="<?= htmlspecialchars($doc_root) ?>"><?= htmlspecialchars($doc_root) ?></span></div>
        <div class="table-row"><span class="label">Server Port</span><span class="value"><?= $_SERVER['SERVER_PORT'] ?? '80' ?></span></div>
        <div class="table-row"><span class="label">Operating System</span><span class="value"><?= htmlspecialchars($os) ?></span></div>
        <div class="table-row"><span class="label">Architecture</span><span class="value"><?= php_uname('m') ?></span></div>
        <div class="table-row"><span class="label">Server Time</span><span class="value"><?= date('Y-m-d H:i:s') ?></span></div>
        <div class="table-row"><span class="label">PHP Extensions</span><span class="value"><?= $ext_count ?> loaded</span></div>
      </div>
    </div>
  </div>

  <!-- Loaded Extensions -->
  <div class="section">
    <div class="section-title">
      <svg class="icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M14.25 6.087c0-.355.186-.676.401-.959.221-.29.349-.634.349-1.003 0-1.036-1.007-1.875-2.25-1.875s-2.25.84-2.25 1.875c0 .369.128.713.349 1.003.215.283.401.604.401.959v0a.64.64 0 0 1-.657.643 48.39 48.39 0 0 1-4.163-.3c.186 1.613.293 3.25.315 4.907a.656.656 0 0 1-.658.663v0c-.355 0-.676-.186-.959-.401a1.647 1.647 0 0 0-1.003-.349c-1.036 0-1.875 1.007-1.875 2.25s.84 2.25 1.875 2.25c.369 0 .713-.128 1.003-.349.283-.215.604-.401.959-.401v0c.31 0 .555.26.532.57a48.039 48.039 0 0 1-.642 5.056c1.518.19 3.058.309 4.616.354a.64.64 0 0 0 .657-.643v0c0-.355-.186-.676-.401-.959a1.647 1.647 0 0 1-.349-1.003c0-1.035 1.008-1.875 2.25-1.875 1.243 0 2.25.84 2.25 1.875 0 .369-.128.713-.349 1.003-.215.283-.4.604-.4.959v0c0 .333.277.599.61.58a48.1 48.1 0 0 0 5.427-.63 48.05 48.05 0 0 0 .582-4.717.532.532 0 0 0-.533-.57v0c-.355 0-.676.186-.959.401-.29.221-.634.349-1.003.349-1.035 0-1.875-1.007-1.875-2.25s.84-2.25 1.875-2.25c.37 0 .713.128 1.003.349.283.215.604.401.96.401v0a.656.656 0 0 0 .658-.663 48.422 48.422 0 0 0-.37-5.36c-1.886.342-3.81.574-5.766.689a.578.578 0 0 1-.61-.58v0Z" /></svg>
      Loaded Extensions
    </div>
    <div class="table-card">
      <div style="padding: 0.75rem 1rem; display: flex; flex-wrap: wrap; gap: 0.375rem;">
        <?php foreach ($loaded_extensions as $ext): ?>
          <span class="badge"><?= htmlspecialchars($ext) ?></span>
        <?php endforeach; ?>
      </div>
    </div>
  </div>

  <!-- PHP Info -->
  <div class="section">
    <div class="section-title">
      <svg class="icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" /></svg>
      PHP Info
    </div>
    <div class="phpinfo-section">
      <button class="phpinfo-toggle" onclick="togglePhpinfo()">
        <span>Full phpinfo() output</span>
        <svg class="arrow" id="phpinfo-arrow" width="16" height="16" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" /></svg>
      </button>
      <div class="phpinfo-content" id="phpinfo-content" style="display: <?= isset($_GET['phpinfo']) ? 'block' : 'none' ?>;">
        <?php
        ob_start();
        phpinfo();
        $phpinfo = ob_get_clean();
        // Extract only the body content
        preg_match('/<body[^>]*>(.*)<\/body>/is', $phpinfo, $matches);
        echo $matches[1] ?? $phpinfo;
        ?>
      </div>
    </div>
  </div>

  <div class="footer">
    Served by <strong>LokcalDev</strong> &mdash; PHP <?= $php_version ?> &middot; <?= htmlspecialchars($nginx_version) ?>
  </div>
</div>

<script>
function togglePhpinfo() {
  const content = document.getElementById('phpinfo-content');
  const arrow = document.getElementById('phpinfo-arrow');
  if (content.style.display === 'none') {
    content.style.display = 'block';
    arrow.style.transform = 'rotate(180deg)';
  } else {
    content.style.display = 'none';
    arrow.style.transform = '';
  }
}
<?php if (isset($_GET['phpinfo'])): ?>
document.getElementById('phpinfo-arrow').style.transform = 'rotate(180deg)';
<?php endif; ?>
</script>
</body>
</html>
