import config

class APIKeyProvider:
	i = 0
	
	def __init__(self):
		self.keys = config.apiKey()

	def apiKey(self):
		key = self.keys[self.i]

		self.i += 1
		
		if self.i > len(self.keys) - 1:
			self.i = 0

		return key

