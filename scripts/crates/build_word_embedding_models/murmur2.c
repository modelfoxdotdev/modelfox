#include <stdint.h>

uint64_t murmur2(const void* key, uint64_t len, uint64_t seed) {
	const uint64_t m = 0xc6a4a7935bd1e995;
	const int r = 47;
	uint64_t h = seed ^ (len * m);
	const uint64_t * data = (const uint64_t *)key;
	const uint64_t * end = data + (len/8);
	while (data != end) {
		uint64_t k = *data++;
		k *= m;
		k ^= k >> r;
		k *= m;
		h ^= k;
		h *= m;
	}
	const unsigned char * data2 = (const unsigned char*)data;
	switch (len & 7) {
    case 7:
      h ^= ((uint64_t)data2[6]) << 48;
      __attribute__ ((fallthrough));
    case 6:
      h ^= ((uint64_t)data2[5]) << 40;
      __attribute__ ((fallthrough));
    case 5:
      h ^= ((uint64_t)data2[4]) << 32;
      __attribute__ ((fallthrough));
    case 4:
      h ^= ((uint64_t)data2[3]) << 24;
      __attribute__ ((fallthrough));
    case 3:
      h ^= ((uint64_t)data2[2]) << 16;
      __attribute__ ((fallthrough));
    case 2:
      h ^= ((uint64_t)data2[1]) << 8;
      __attribute__ ((fallthrough));
    case 1:
      h ^= ((uint64_t)data2[0]);
      __attribute__ ((fallthrough));
    default:
      h *= m;
	}
	h ^= h >> r;
	h *= m;
	h ^= h >> r;
	return h;
}
