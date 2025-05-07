import { NextResponse } from "next/server";
import { getPaginatedPosts } from "@/lib/data";

export async function GET(request: Request) {
  try {
    // Get query parameters
    const url = new URL(request.url);
    const page = parseInt(url.searchParams.get("page") || "1");
    const limit = parseInt(url.searchParams.get("limit") || "10");
    
    // Simulate a slight delay to demonstrate loading states
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Get paginated posts
    const { posts, hasMore } = getPaginatedPosts(page, limit);
    
    return NextResponse.json({ posts, hasMore });
  } catch (error) {
    console.error("Error fetching posts:", error);
    return NextResponse.json(
      { error: "Failed to fetch posts" },
      { status: 500 }
    );
  }
}

