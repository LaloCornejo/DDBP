// Mock data for our application

// Users
export const users = [
  {
    _id: "f9b3c87d-1a2b-4c3d-8e4f-5a6b7c8d9e0f",
    name: "John Doe",
    username: "johndoe",
    email: "john@example.com",
    profile_picture_url: "https://randomuser.me/api/portraits/men/1.jpg",
    bio: "Software engineer and tech enthusiast.",
    postCount: 2,
    join_date: "2025-01-15T10:00:00Z"
  },
  {
    _id: "a1b2c3d4-e5f6-7g8h-9i0j-1k2l3m4n5o6p",
    name: "Jane Smith",
    username: "janesmith",
    email: "jane@example.com",
    profile_picture_url: "https://randomuser.me/api/portraits/women/2.jpg",
    bio: "UX designer and digital artist.",
    postCount: 1,
    join_date: "2025-02-20T14:30:00Z"
  },
  {
    _id: "7p8o9i0u-1y2t3r4e-5w6q7a8s-9d0f1g2h",
    name: "Alex Johnson",
    username: "alexj",
    email: "alex@example.com",
    profile_picture_url: "https://randomuser.me/api/portraits/men/3.jpg",
    bio: "Product manager and entrepreneur.",
    postCount: 1,
    join_date: "2025-03-10T09:15:00Z"
  }
];

// Posts
export const posts = [
  {
    _id: "12037d5d-afe1-4ad5-ad2c-71f430770ef5",
    title: "Getting Started with Next.js",
    content: "Next.js is a React framework that enables you to build server-side rendered applications. It offers a great developer experience with features like file-system based routing, API routes, and built-in CSS support. In this post, we'll explore how to get started with Next.js and create your first application. We'll cover the basic concepts, project structure, and deployment options. By the end of this tutorial, you'll have a solid understanding of how Next.js works and how it can benefit your projects.",
    user_id: "f9b3c87d-1a2b-4c3d-8e4f-5a6b7c8d9e0f",
    comment_count: 2,
    created_at: "2025-04-30T12:00:00Z"
  },
  {
    _id: "32d1d88c-09d1-4e0d-a148-385d305bd554",
    title: "The Power of Shadcn UI Components",
    content: "Shadcn UI is a collection of beautifully designed, accessible, and customizable components built on top of Tailwind CSS. In this post, we'll look at how to leverage these components to create stunning user interfaces quickly. We'll demonstrate how to install and configure the components, how to customize them to match your brand, and how to combine them to create complex UI patterns. By using a component library like Shadcn UI, you can save time and ensure consistency across your application.",
    user_id: "f9b3c87d-1a2b-4c3d-8e4f-5a6b7c8d9e0f",
    comment_count: 1,
    created_at: "2025-05-01T15:30:00Z"
  },
  {
    _id: "f1f918b1-ab23-4f59-8a9a-d13bb7cf43e2",
    title: "Modern Web Design Trends in 2025",
    content: "Web design is constantly evolving, with new trends emerging every year. In 2025, we're seeing a shift towards more immersive experiences, minimalistic interfaces, and personalized content. This post explores the top design trends that are shaping the web this year. From neomorphism to micro-interactions, from dark mode to voice user interfaces, we'll examine what makes these trends effective and how you can incorporate them into your projects.",
    user_id: "a1b2c3d4-e5f6-7g8h-9i0j-1k2l3m4n5o6p",
    comment_count: 3,
    created_at: "2025-05-02T09:15:00Z"
  },
  {
    _id: "92cb507c-4358-49e5-937c-18596981f822",
    title: "Building a Scalable API with Node.js",
    content: "Creating a scalable and maintainable API is essential for modern web applications. In this comprehensive guide, we'll walk through the process of building a robust API using Node.js and Express. We'll cover topics such as API design principles, authentication, error handling, testing, and performance optimization. By the end of this tutorial, you'll have the knowledge to create APIs that can handle high traffic and scale with your application's needs.",
    user_id: "7p8o9i0u-1y2t3r4e-5w6q7a8s-9d0f1g2h",
    comment_count: 0,
    created_at: "2025-05-03T18:45:00Z"
  }
];

// Comments
export const comments = [
  {
    _id: "b7c8d9e0-f1a2-b3c4-d5e6-f7a8b9c0d1e2",
    post_id: "12037d5d-afe1-4ad5-ad2c-71f430770ef5",
    content: "Great introduction to Next.js! I've been looking for a comprehensive guide like this.",
    user_id: "a1b2c3d4-e5f6-7g8h-9i0j-1k2l3m4n5o6p",
    created_at: "2025-04-30T14:23:00Z"
  },
  {
    _id: "3f4g5h6i-7j8k-9l0m-1n2o-3p4q5r6s7t8u",
    post_id: "12037d5d-afe1-4ad5-ad2c-71f430770ef5",
    content: "Thanks for sharing this. I'm just getting started with Next.js and found this really helpful.",
    user_id: "7p8o9i0u-1y2t3r4e-5w6q7a8s-9d0f1g2h",
    created_at: "2025-04-30T18:15:00Z"
  },
  {
    _id: "9v0w1x2y-3z4a-5b6c-7d8e-9f0g1h2i3j4k",
    post_id: "32d1d88c-09d1-4e0d-a148-385d305bd554",
    content: "I've been using Shadcn UI for my recent project and it's been a game changer. The components are so well designed!",
    user_id: "a1b2c3d4-e5f6-7g8h-9i0j-1k2l3m4n5o6p",
    created_at: "2025-05-01T16:42:00Z"
  },
  {
    _id: "5l6m7n8o-9p0q-1r2s-3t4u-5v6w7x8y9z0a",
    post_id: "f1f918b1-ab23-4f59-8a9a-d13bb7cf43e2",
    content: "Really insightful analysis of current design trends. The section about micro-interactions was particularly helpful.",
    user_id: "f9b3c87d-1a2b-4c3d-8e4f-5a6b7c8d9e0f",
    created_at: "2025-05-02T10:30:00Z"
  },
  {
    _id: "2b3c4d5e-6f7g-8h9i-0j1k-2l3m4n5o6p7q",
    post_id: "f1f918b1-ab23-4f59-8a9a-d13bb7cf43e2",
    content: "Great overview of the latest trends. I'm particularly excited about the voice UI developments.",
    user_id: "7p8o9i0u-1y2t3r4e-5w6q7a8s-9d0f1g2h",
    created_at: "2025-05-02T11:15:00Z"
  },
  {
    _id: "8r9s0t1u-2v3w-4x5y-6z7a-8b9c0d1e2f3g",
    post_id: "f1f918b1-ab23-4f59-8a9a-d13bb7cf43e2",
    content: "The section about dark mode implementation was exactly what I needed. Thanks for sharing!",
    user_id: "a1b2c3d4-e5f6-7g8h-9i0j-1k2l3m4n5o6p",
    created_at: "2025-05-02T13:45:00Z"
  }
];

// Helper functions to simulate API operations
export async function getUserById(id: string) {
  const user = users.find(user => user._id === id);
  if (!user) return null;
  return Promise.resolve(user);
}

export async function getPostById(id: string) {
  const post = posts.find(post => post._id === id);
  if (!post) return null;
  return Promise.resolve(post);
}

export async function getCommentsByPostId(postId: string) {
  const postComments = comments.filter(comment => comment.post_id === postId);
  return Promise.resolve(postComments);
}

export async function getPostsByAuthorId(authorId: string) {
  const authorPosts = posts.filter(post => post.user_id === authorId);
  return Promise.resolve(authorPosts);
}

export async function getPaginatedPosts(page: number = 1, limit: number = 10) {
  const startIndex = (page - 1) * limit;
  const endIndex = page * limit;
  
  const paginatedPosts = await Promise.all(posts
    .slice(startIndex, endIndex)
    .map(async post => {
      const author = await getUserById(post.user_id);
      return {
        id: post._id,
        title: post.title,
        content: post.content,
        author: {
          id: author?._id,
          name: author?.name || author?.username,
          image: author?.profile_picture_url
        },
        timestamp: post.created_at,
        commentCount: post.comment_count
      };
    }));
  
  return {
    posts: paginatedPosts,
    hasMore: endIndex < posts.length
  };
}

export async function enrichPostWithDetails(post: typeof posts[0]) {
  const author = await getUserById(post.user_id);
  const postComments = await getCommentsByPostId(post._id);
  
  const enrichedComments = await Promise.all(postComments.map(async comment => {
    const commentAuthor = await getUserById(comment.user_id);
    return {
      id: comment._id,
      content: comment.content,
      timestamp: comment.created_at,
      author: {
        id: commentAuthor?._id,
        name: commentAuthor?.name || commentAuthor?.username,
        image: commentAuthor?.profile_picture_url
      }
    };
  }));
  
  return {
    id: post._id,
    title: post.title,
    content: post.content,
    author: {
      id: author?._id,
      name: author?.name || author?.username,
      image: author?.profile_picture_url
    },
    comments: enrichedComments,
    timestamp: post.created_at,
    commentCount: post.comment_count || enrichedComments.length
  };
}

