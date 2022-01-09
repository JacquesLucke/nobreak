#include <array>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <vector>

enum class Endian {
#ifdef _WIN32
  little = 0,
  big = 1,
  native = little,
#else
  little = __ORDER_LITTLE_ENDIAN__,
  big = __ORDER_BIG_ENDIAN__,
  native = __BYTE_ORDER__,
#endif
  network = big,
};

static int32_t swap_byte_order(const int32_t x) {
  return ((x & 0x000000ff) << 24) | ((x & 0x0000ff00) << 8) |
         ((x & 0x00ff0000) >> 8) | ((x & 0xff000000) >> 24);
}

static void encode_int(int32_t value, std::vector<uint8_t> &r_result) {
  if constexpr (Endian::native != Endian::network) {
    value = swap_byte_order(value);
  }
  std::array<uint8_t, 4> bytes;
  memcpy(bytes.data(), &value, sizeof(int32_t));
  r_result.insert(r_result.end(), bytes.begin(), bytes.end());
}

static int32_t decode_int(const std::vector<uint8_t> &buffer, int offset) {
  int32_t value;
  memcpy(&value, buffer.data() + offset, sizeof(int32_t));
  if constexpr (Endian::native != Endian::network) {
    value = swap_byte_order(value);
  }
  return value;
}

static void encode_full_key(const std::vector<std::string> &full_key,
                            std::vector<uint8_t> &r_result) {
  encode_int(static_cast<int32_t>(full_key.size()), r_result);
  for (const std::string &key : full_key) {
    encode_int(static_cast<int32_t>(key.size()), r_result);
    r_result.insert(r_result.end(), key.begin(), key.end());
  }
}

static std::vector<std::string>
decode_full_key(const std::vector<uint8_t> &buffer) {
  std::vector<std::string> full_key;
  int offset = 0;
  const int key_amount = decode_int(buffer, offset);
  offset += sizeof(int32_t);
  for (int i = 0; i < key_amount; i++) {
    const int key_length = decode_int(buffer, offset);
    offset += sizeof(int32_t);

    std::string key(key_length, '!');
    memcpy(key.data(), buffer.data() + offset, key_length);
    offset += key_length;

    full_key.push_back(std::move(key));
  }
  return full_key;
}

int main(int argc, char const *argv[]) {

  std::vector<uint8_t> result;
  std::vector<std::string> full_key = {"test", "a", "b", "c"};
  encode_full_key(full_key, result);
  for (const uint8_t b : result) {
    std::cout << (int)b << " ";
  }
  std::cout << "\n";

  std::vector<std::string> decoded = decode_full_key(result);
  for (const std::string &key : decoded) {
    std::cout << key << ", ";
  }
  std::cout << "\n";

  /* code */
  return 0;
}
