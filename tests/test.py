import smtplib

# Establish a connection to the SMTP server
server = smtplib.SMTP('127.0.0.1', 5878)

# Send the HELO command
server.ehlo()

# Set the sender and recipient
sender = 'sender@example.com'
recipient = 'recipient@example.com'

# Create the email
subject = 'Test Email'
body = 'This is a test email.'
email = f'Subject: {subject}\n\n{body}'

# Send the email
server.sendmail(sender, recipient, email)

# Close the connection to the server
server.quit()