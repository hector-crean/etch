// Example showing how links will be generated with the Button wrapper

// When as_button is false (default behavior):
// <Link href="/dashboard">Go to Dashboard</Link>

// When as_button is true:
// <Button asChild>
//   <Link href="/login">Login</Link>
// </Button>

// When as_button is true with variant and size:
// <Button asChild variant="outline" size="lg">
//   <Link href="/signup">Sign Up</Link>
// </Button>

// This works for all routing libraries:
// NextJS: <Button asChild><Link href="/path">Text</Link></Button>
// Wouter: <Button asChild><Link href="/path">Text</Link></Button>
// React Router: <Button asChild><Link to="/path">Text</Link></Button>
// Native: <Button asChild><a href="/path">Text</a></Button>

export default function LinkButtonExample() {
  return (
    <div className="space-y-4">
      <h2>Link Button Examples</h2>
      
      {/* Regular link */}
      <Link href="/dashboard">Go to Dashboard</Link>
      
      {/* Button-styled links */}
      <Button asChild>
        <Link href="/login">Login</Link>
      </Button>
      
      <Button asChild variant="outline" size="lg">
        <Link href="/signup">Sign Up</Link>
      </Button>
      
      <Button asChild variant="destructive">
        <Link href="/logout">Logout</Link>
      </Button>
    </div>
  );
}
