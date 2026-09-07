---
let user_id = params.id;
let user = db_first("SELECT * FROM users WHERE id = ?", [user_id]);
let exists = user != ();
---
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>User Profile - Carbon</title>
  <link rel="stylesheet" href="/app.css">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;600;700;800&display=swap">
</head>
<body>
  <div class="container" style="max-width: 600px; padding-top: 60px;">
    <div class="card">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
        <a href="/" class="user-link">← Back to Dashboard</a>
        <span class="badge">DYNAMIC ROUTE: /users/[id]</span>
      </div>

      {% if exists %}
        <div style="text-align: center; padding: 20px 0;">
          <div style="width: 72px; height: 72px; background: #38bdf8; color: #090d16; border-radius: 50%; display: grid; place-content: center; font-size: 32px; font-weight: bold; margin: 0 auto 16px;">
            ⚡
          </div>
          <h1 style="margin: 0 0 8px;">{{ user.name }}</h1>
          <p style="color: #94a3b8; margin: 0;">{{ user.email }}</p>
        </div>

        <div style="background: #090d16; padding: 16px; border-radius: 8px; margin-top: 20px;">
          <table style="width: 100%; color: #e2e8f0; font-size: 14px;">
            <tr>
              <td style="color: #64748b; padding: 8px 0;">User ID:</td>
              <td style="font-weight: bold;">#{{ user.id }}</td>
            </tr>
            <tr>
              <td style="color: #64748b; padding: 8px 0;">Role:</td>
              <td><span class="role-tag">{{ user.role }}</span></td>
            </tr>
            <tr>
              <td style="color: #64748b; padding: 8px 0;">Created At:</td>
              <td style="color: #a78bfa;">{{ user.created_at }}</td>
            </tr>
          </table>
        </div>
      {% else %}
        <div style="text-align: center; padding: 40px 0;">
          <h2 style="color: #f43f5e;">User Not Found</h2>
          <p style="color: #94a3b8;">User #{{ user_id }} does not exist in SQLite database.</p>
        </div>
      {% endif %}
    </div>
  </div>
</body>
</html>
