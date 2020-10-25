import config

class APIKeyProvider:
	i = 0

	def apiKey(self):
		keys = config.apiKey()

		key = keys[self.i]

		self.i += 1
		
		if self.i > len(keys) - 1:
			self.i = 0

		return key

