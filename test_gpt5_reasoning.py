#!/usr/bin/env python3
"""
Test GPT-5 with reasoning_effort to see if reasoning chunks are separate
"""
import requests
import json
import sys

API_KEY = "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"

url = "https://api.openai.com/v1/chat/completions"

payload = {
    "model": "gpt-5",
    "messages": [
        {"role": "user", "content": "Calculate 23 * 17"}
    ],
    "stream": True,
    "reasoning_effort": "high"
}

headers = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {API_KEY}"
}

print("🔬 Testing GPT-5 with reasoning_effort=high + streaming\n")
print("="*80)

response = requests.post(url, headers=headers, json=payload, stream=True)

chunk_count = 0
seen_fields = set()

for line in response.iter_lines():
    if line:
        line_str = line.decode('utf-8')
        
        if line_str.startswith('data: '):
            data_str = line_str[6:]  # Remove 'data: '
            
            if data_str == '[DONE]':
                print("\n✅ Stream complete!")
                break
            
            try:
                data = json.loads(data_str)
                chunk_count += 1
                
                if 'choices' in data and len(data['choices']) > 0:
                    choice = data['choices'][0]
                    delta = choice.get('delta', {})
                    
                    # Track all unique fields seen
                    for key in delta.keys():
                        if key not in seen_fields:
                            seen_fields.add(key)
                            print(f"\n🆕 NEW FIELD DISCOVERED: '{key}'")
                    
                    # Print first 10 chunks in detail
                    if chunk_count <= 10:
                        print(f"\n📦 Chunk #{chunk_count}")
                        print(f"   Delta keys: {list(delta.keys())}")
                        print(f"   Delta: {json.dumps(delta, indent=6)}")
                    
                    # Check for reasoning-specific fields
                    if 'reasoning' in delta or 'reasoning_content' in delta:
                        print(f"\n🎯 REASONING FIELD FOUND IN CHUNK #{chunk_count}!")
                        print(f"   Content: {delta}")
                    
                    # Print content chunks
                    if 'content' in delta and delta['content']:
                        print(delta['content'], end='', flush=True)
            
            except json.JSONDecodeError:
                pass

print(f"\n\n{'='*80}")
print(f"📊 Summary:")
print(f"   Total chunks: {chunk_count}")
print(f"   Unique delta fields seen: {sorted(seen_fields)}")
print(f"\n💡 Analysis:")

if 'reasoning' in seen_fields or 'reasoning_content' in seen_fields:
    print("   ✅ REASONING CHUNKS ARE SEPARATE!")
else:
    print("   ❌ No separate reasoning field found")
    print("   ℹ️  Reasoning is mixed with content (like GPT-4)")
